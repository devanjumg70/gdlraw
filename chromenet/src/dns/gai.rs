//! System DNS resolver using getaddrinfo.
//!
//! This resolver uses the operating system's native DNS resolution via
//! `getaddrinfo`, executed in a thread pool to avoid blocking the async runtime.
//!
//! # When to Use
//!
//! - When you need to respect system DNS configuration (/etc/resolv.conf, etc.)
//! - When DoH/DoT is not required
//! - As a fallback when hickory-dns is not available

use super::{Addrs, Name, Resolve, Resolving};
use crate::base::neterror::NetError;
use std::{
    io,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, ToSocketAddrs},
};

/// System DNS resolver using `getaddrinfo` in a thread pool.
///
/// This resolver wraps the standard library's `ToSocketAddrs` trait and
/// executes resolution in `tokio::task::spawn_blocking` to avoid blocking
/// the async runtime.
///
/// # Performance
///
/// Each resolution spawns a blocking task. For high-throughput scenarios,
/// consider using `HickoryResolver` which is fully async.
#[derive(Clone, Debug, Default)]
pub struct GaiResolver;

impl GaiResolver {
    /// Creates a new `GaiResolver`.
    pub fn new() -> Self {
        Self
    }
}

impl Resolve for GaiResolver {
    fn resolve(&self, name: Name) -> Resolving {
        Box::pin(async move {
            let host = name.as_str().to_string();
            let domain = host.clone();

            let result = tokio::task::spawn_blocking(move || {
                tracing::debug!(host = %host, "resolving via getaddrinfo");
                (host.as_str(), 0u16)
                    .to_socket_addrs()
                    .map(|iter| iter.collect::<Vec<_>>())
            })
            .await;

            // Handle task join error (cancellation, panic)
            let addrs = result
                .map_err(|e| {
                    tracing::error!(error = %e, "DNS resolution task failed");
                    NetError::NameNotResolved
                })?
                .map_err(|e| {
                    tracing::debug!(domain = %domain, error = %e, "DNS resolution failed");
                    NetError::NameNotResolvedFor {
                        domain: domain.clone(),
                        source: std::sync::Arc::new(e),
                    }
                })?;

            if addrs.is_empty() {
                return Err(NetError::NameNotResolvedFor {
                    domain,
                    source: std::sync::Arc::new(io::Error::new(
                        io::ErrorKind::NotFound,
                        "No addresses returned by getaddrinfo",
                    )),
                });
            }

            tracing::debug!(domain = %domain, count = addrs.len(), "DNS resolution complete");
            Ok(Box::new(addrs.into_iter()) as Addrs)
        })
    }
}

/// Utility for parsing IP address strings directly.
///
/// Bypasses DNS resolution if the host is already an IP address.
pub struct SocketAddrs {
    addrs: Vec<SocketAddr>,
}

impl SocketAddrs {
    /// Creates a new `SocketAddrs` from a vector.
    pub fn new(addrs: Vec<SocketAddr>) -> Self {
        Self { addrs }
    }

    /// Attempts to parse a host string as an IP address.
    ///
    /// Returns `Some` if the host is a valid IPv4 or IPv6 address,
    /// `None` if it's a hostname that requires DNS resolution.
    pub fn try_parse(host: &str, port: u16) -> Option<Self> {
        // Try IPv4
        if let Ok(addr) = host.parse::<Ipv4Addr>() {
            return Some(Self {
                addrs: vec![SocketAddr::V4(SocketAddrV4::new(addr, port))],
            });
        }

        // Try IPv6
        if let Ok(addr) = host.parse::<Ipv6Addr>() {
            return Some(Self {
                addrs: vec![SocketAddr::V6(SocketAddrV6::new(addr, port, 0, 0))],
            });
        }

        None
    }

    /// Returns true if no addresses are available.
    pub fn is_empty(&self) -> bool {
        self.addrs.is_empty()
    }

    /// Returns the number of addresses.
    pub fn len(&self) -> usize {
        self.addrs.len()
    }

    /// Splits addresses by IP version preference.
    ///
    /// Used for Happy Eyeballs (RFC 8305) implementation.
    /// Returns (preferred, fallback) based on first address family or local bindings.
    pub fn split_by_preference(
        self,
        local_ipv4: Option<Ipv4Addr>,
        local_ipv6: Option<Ipv6Addr>,
    ) -> (Self, Self) {
        match (local_ipv4, local_ipv6) {
            // Only IPv4 local address: filter to IPv4 only
            (Some(_), None) => {
                let addrs = self.addrs.into_iter().filter(|a| a.is_ipv4()).collect();
                (Self { addrs }, Self { addrs: vec![] })
            }
            // Only IPv6 local address: filter to IPv6 only
            (None, Some(_)) => {
                let addrs = self.addrs.into_iter().filter(|a| a.is_ipv6()).collect();
                (Self { addrs }, Self { addrs: vec![] })
            }
            // Both or neither: prefer first family, fallback to other
            _ => {
                let prefer_v6 = self.addrs.first().map(|a| a.is_ipv6()).unwrap_or(false);
                let (preferred, fallback): (Vec<_>, Vec<_>) = self
                    .addrs
                    .into_iter()
                    .partition(|a| a.is_ipv6() == prefer_v6);
                (Self { addrs: preferred }, Self { addrs: fallback })
            }
        }
    }
}

impl Iterator for SocketAddrs {
    type Item = SocketAddr;

    fn next(&mut self) -> Option<Self::Item> {
        if self.addrs.is_empty() {
            None
        } else {
            Some(self.addrs.remove(0))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    #[test]
    fn test_socket_addrs_try_parse_ipv4() {
        let result = SocketAddrs::try_parse("127.0.0.1", 8080);
        assert!(result.is_some());

        let addrs = result.unwrap();
        assert_eq!(addrs.len(), 1);
        assert_eq!(
            addrs.addrs[0],
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080)
        );
    }

    #[test]
    fn test_socket_addrs_try_parse_ipv6() {
        let result = SocketAddrs::try_parse("::1", 443);
        assert!(result.is_some());

        let addrs = result.unwrap();
        assert_eq!(addrs.len(), 1);
        assert!(addrs.addrs[0].is_ipv6());
    }

    #[test]
    fn test_socket_addrs_try_parse_hostname() {
        let result = SocketAddrs::try_parse("example.com", 80);
        assert!(result.is_none());
    }

    #[test]
    fn test_split_by_preference_ipv4_only() {
        let addrs = SocketAddrs::new(vec![
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4)), 0),
            SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 0),
        ]);

        let (preferred, fallback) = addrs.split_by_preference(Some(Ipv4Addr::LOCALHOST), None);

        assert_eq!(preferred.len(), 1);
        assert!(preferred.addrs[0].is_ipv4());
        assert!(fallback.is_empty());
    }

    #[test]
    fn test_split_by_preference_mixed() {
        let addrs = SocketAddrs::new(vec![
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4)), 0),
            SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 0),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(5, 6, 7, 8)), 0),
        ]);

        let (preferred, fallback) = addrs.split_by_preference(None, None);

        // First is IPv4, so IPv4 is preferred
        assert_eq!(preferred.len(), 2);
        assert!(preferred.addrs.iter().all(|a| a.is_ipv4()));
        assert_eq!(fallback.len(), 1);
        assert!(fallback.addrs[0].is_ipv6());
    }

    #[tokio::test]
    async fn test_gai_resolver_localhost() {
        let resolver = GaiResolver::new();
        let result = resolver.resolve(Name::new("localhost")).await;

        // localhost should always resolve
        assert!(result.is_ok());
        let addrs: Vec<_> = result.unwrap().collect();
        assert!(!addrs.is_empty());
    }
}
