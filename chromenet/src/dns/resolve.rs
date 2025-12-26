//! Core DNS resolution types and traits.
//!
//! This module defines the `Resolve` trait and supporting types that form
//! the foundation of the DNS abstraction layer.

use crate::base::neterror::NetError;
use std::{
    borrow::Cow, collections::HashMap, fmt, future::Future, net::SocketAddr, pin::Pin, sync::Arc,
};

/// A domain name to resolve into IP addresses.
///
/// This is a lightweight wrapper around a hostname string that provides
/// a type-safe way to pass domain names to resolvers.
#[derive(Clone, Hash, Eq, PartialEq)]
pub struct Name {
    host: Box<str>,
}

impl Name {
    /// Creates a new [`Name`] from any string-like type.
    #[inline]
    pub fn new(host: impl Into<Box<str>>) -> Self {
        Self { host: host.into() }
    }

    /// View the hostname as a string slice.
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.host
    }
}

impl From<&str> for Name {
    fn from(value: &str) -> Self {
        Name::new(value)
    }
}

impl From<String> for Name {
    fn from(value: String) -> Self {
        Name::new(value)
    }
}

impl fmt::Debug for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.host, f)
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.host, f)
    }
}

/// Alias for an `Iterator` trait object over `SocketAddr`.
pub type Addrs = Box<dyn Iterator<Item = SocketAddr> + Send>;

/// Alias for the `Future` type returned by a DNS resolver.
pub type Resolving = Pin<Box<dyn Future<Output = Result<Addrs, NetError>> + Send>>;

/// Trait for DNS resolution.
///
/// This is the core abstraction for DNS resolvers in chromenet, equivalent
/// to Chromium's `HostResolver`. Implementations must be thread-safe.
///
/// # Design Notes
///
/// - Resolution is assumed to always be ready (no backpressure).
/// - Uses `&self` for concurrent resolution without mutable access.
/// - Returns boxed futures for trait object compatibility.
pub trait Resolve: Send + Sync {
    /// Resolves a domain name to IP addresses.
    ///
    /// The returned addresses will have port 0; callers should set the
    /// appropriate port based on the target service.
    fn resolve(&self, name: Name) -> Resolving;
}

/// Blanket implementation for Arc-wrapped resolvers.
impl<R: Resolve + ?Sized> Resolve for Arc<R> {
    fn resolve(&self, name: Name) -> Resolving {
        (**self).resolve(name)
    }
}

/// DNS resolver wrapper that supports hostname overrides.
///
/// This resolver first checks a map of hostname-to-address overrides before
/// falling back to the underlying resolver. Useful for:
/// - Testing without real DNS
/// - Forcing specific IPs for certain domains
/// - Local development with custom hostnames
///
/// # Example
///
/// ```rust,ignore
/// use chromenet::dns::{DnsResolverWithOverrides, HickoryResolver, Name};
/// use std::collections::HashMap;
/// use std::net::SocketAddr;
///
/// let mut overrides = HashMap::new();
/// overrides.insert(
///     "api.local".into(),
///     vec!["127.0.0.1:0".parse().unwrap()],
/// );
///
/// let resolver = DnsResolverWithOverrides::new(
///     Arc::new(HickoryResolver::new()),
///     overrides,
/// );
/// ```
pub struct DnsResolverWithOverrides {
    inner: Arc<dyn Resolve>,
    overrides: Arc<HashMap<Cow<'static, str>, Vec<SocketAddr>>>,
}

impl DnsResolverWithOverrides {
    /// Creates a new resolver with the given overrides.
    ///
    /// # Arguments
    ///
    /// * `inner` - The fallback resolver for non-overridden hostnames.
    /// * `overrides` - Map of hostnames to their resolved addresses.
    pub fn new(
        inner: Arc<dyn Resolve>,
        overrides: HashMap<Cow<'static, str>, Vec<SocketAddr>>,
    ) -> Self {
        Self {
            inner,
            overrides: Arc::new(overrides),
        }
    }

    /// Returns the number of configured overrides.
    pub fn override_count(&self) -> usize {
        self.overrides.len()
    }
}

impl Resolve for DnsResolverWithOverrides {
    fn resolve(&self, name: Name) -> Resolving {
        // Check overrides first
        if let Some(addrs) = self.overrides.get(name.as_str()) {
            let addrs: Addrs = Box::new(addrs.clone().into_iter());
            return Box::pin(std::future::ready(Ok(addrs)));
        }
        // Fall back to inner resolver
        self.inner.resolve(name)
    }
}

impl fmt::Debug for DnsResolverWithOverrides {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DnsResolverWithOverrides")
            .field("override_count", &self.overrides.len())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_name_from_str() {
        let name = Name::from("example.com");
        assert_eq!(name.as_str(), "example.com");
        assert_eq!(name.to_string(), "example.com");
    }

    #[test]
    fn test_name_from_string() {
        let domain = String::from("test.example.com");
        let name = Name::from(domain);
        assert_eq!(name.as_str(), "test.example.com");
    }

    #[test]
    fn test_name_equality() {
        let name1 = Name::new("example.com");
        let name2 = Name::new("example.com");
        let name3 = Name::new("other.com");

        assert_eq!(name1, name2);
        assert_ne!(name1, name3);
    }

    #[test]
    fn test_name_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(Name::new("example.com"));
        set.insert(Name::new("example.com")); // Duplicate

        assert_eq!(set.len(), 1);
    }

    struct MockResolver {
        response: Vec<SocketAddr>,
    }

    impl Resolve for MockResolver {
        fn resolve(&self, _name: Name) -> Resolving {
            let addrs = self.response.clone();
            Box::pin(async move { Ok(Box::new(addrs.into_iter()) as Addrs) })
        }
    }

    #[tokio::test]
    async fn test_override_resolver_hit() {
        let mock = Arc::new(MockResolver {
            response: vec![SocketAddr::new(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)), 0)],
        });

        let mut overrides = HashMap::new();
        overrides.insert(
            Cow::Borrowed("override.local"),
            vec![SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0)],
        );

        let resolver = DnsResolverWithOverrides::new(mock, overrides);
        let addrs: Vec<_> = resolver
            .resolve(Name::new("override.local"))
            .await
            .unwrap()
            .collect();

        assert_eq!(addrs.len(), 1);
        assert_eq!(addrs[0].ip(), IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
    }

    #[tokio::test]
    async fn test_override_resolver_miss() {
        let mock = Arc::new(MockResolver {
            response: vec![SocketAddr::new(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)), 0)],
        });

        let overrides = HashMap::new();
        let resolver = DnsResolverWithOverrides::new(mock, overrides);

        let addrs: Vec<_> = resolver
            .resolve(Name::new("not-overridden.com"))
            .await
            .unwrap()
            .collect();

        assert_eq!(addrs.len(), 1);
        assert_eq!(addrs[0].ip(), IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)));
    }
}
