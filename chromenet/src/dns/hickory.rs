//! Async DNS resolver using hickory-dns.
//!
//! This resolver provides fully async DNS resolution with support for:
//! - DNS-over-HTTPS (DoH)
//! - DNS-over-TLS (DoT)
//! - System DNS configuration auto-detection
//! - Happy Eyeballs (IPv4 + IPv6 lookup)
//!
//! # Performance
//!
//! Unlike `GaiResolver`, this resolver is fully async and doesn't require
//! spawning blocking tasks. It maintains connection pools to DNS servers
//! for better performance under load.

use super::{Addrs, Name, Resolve, Resolving};
use crate::base::neterror::NetError;
use hickory_resolver::{
    config::{LookupIpStrategy, ResolverConfig},
    name_server::TokioConnectionProvider,
    TokioResolver,
};
use std::{net::SocketAddr, sync::LazyLock};

/// Async DNS resolver backed by hickory-dns.
///
/// This resolver is lazily initialized on first use and shared across
/// all instances via a static `LazyLock`. It automatically configures
/// itself based on the system's DNS settings.
///
/// # Features
///
/// - Fully async (no blocking threads)
/// - Automatic system configuration detection
/// - IPv4 and IPv6 dual-stack resolution
/// - Connection pooling to DNS servers
///
/// # Example
///
/// ```rust,ignore
/// use chromenet::dns::{HickoryResolver, Name, Resolve};
///
/// let resolver = HickoryResolver::new();
/// let addrs = resolver.resolve(Name::new("example.com")).await?;
/// ```
#[derive(Debug, Clone)]
pub struct HickoryResolver {
    resolver: &'static LazyLock<TokioResolver>,
}

impl HickoryResolver {
    /// Creates a new `HickoryResolver`.
    ///
    /// The underlying resolver is lazily initialized on first DNS query.
    /// It will attempt to read system DNS configuration; if that fails,
    /// it falls back to sensible defaults.
    pub fn new() -> Self {
        static RESOLVER: LazyLock<TokioResolver> = LazyLock::new(|| {
            let mut builder = match TokioResolver::builder_tokio() {
                Ok(builder) => {
                    tracing::debug!("Using system DNS configuration");
                    builder
                }
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        "Failed to read system DNS config, using defaults"
                    );
                    TokioResolver::builder_with_config(
                        ResolverConfig::default(),
                        TokioConnectionProvider::default(),
                    )
                }
            };

            // Enable dual-stack for Happy Eyeballs
            builder.options_mut().ip_strategy = LookupIpStrategy::Ipv4AndIpv6;

            builder.build()
        });

        Self {
            resolver: &RESOLVER,
        }
    }
}

impl Default for HickoryResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl Resolve for HickoryResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let resolver = self.clone();
        Box::pin(async move {
            let domain = name.as_str();
            tracing::debug!(domain = %domain, "resolving via hickory-dns");

            let lookup = resolver.resolver.lookup_ip(domain).await.map_err(|e| {
                tracing::debug!(domain = %domain, error = %e, "hickory-dns lookup failed");
                NetError::NameNotResolvedFor {
                    domain: domain.to_string(),
                    source: std::sync::Arc::new(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        e.to_string(),
                    )),
                }
            })?;

            let addrs: Vec<SocketAddr> = lookup.iter().map(|ip| SocketAddr::new(ip, 0)).collect();

            if addrs.is_empty() {
                return Err(NetError::NameNotResolvedFor {
                    domain: domain.to_string(),
                    source: std::sync::Arc::new(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "No addresses returned",
                    )),
                });
            }

            tracing::debug!(domain = %domain, count = addrs.len(), "hickory-dns resolution complete");
            Ok(Box::new(addrs.into_iter()) as Addrs)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hickory_resolver_known_domain() {
        let resolver = HickoryResolver::new();
        let result = resolver.resolve(Name::new("localhost")).await;

        // localhost should resolve on any system
        assert!(result.is_ok());
        let addrs: Vec<_> = result.unwrap().collect();
        assert!(!addrs.is_empty());
    }

    #[tokio::test]
    async fn test_hickory_resolver_invalid_domain() {
        let resolver = HickoryResolver::new();
        let result = resolver
            .resolve(Name::new("this-domain-definitely-does-not-exist.invalid"))
            .await;

        assert!(result.is_err());
        let err = result.err().expect("Should have error");
        match err {
            NetError::NameNotResolvedFor { domain, .. } => {
                assert_eq!(domain, "this-domain-definitely-does-not-exist.invalid");
            }
            _ => panic!("Unexpected error type"),
        }
    }

    #[test]
    fn test_hickory_resolver_is_clone() {
        let r1 = HickoryResolver::new();
        let r2 = r1.clone();
        // Both should point to the same static resolver
        assert!(std::ptr::eq(r1.resolver, r2.resolver));
    }
}
