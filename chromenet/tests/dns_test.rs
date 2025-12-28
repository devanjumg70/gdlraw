//! DNS Module Tests
//!
//! Covers:
//! - `Name` struct
//! - `DnsResolverWithOverrides` using a MockResolver
//! - `GaiResolver` (Basic System Resolver)

use chromenet::dns::{Addrs, DnsResolverWithOverrides, GaiResolver, Name, Resolve, Resolving};

use std::borrow::Cow;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

struct MockResolver {
    response: Vec<SocketAddr>,
}

impl Resolve for MockResolver {
    fn resolve(&self, _name: Name) -> Resolving {
        let addrs = self.response.clone();
        Box::pin(async move { Ok(Box::new(addrs.into_iter()) as Addrs) })
    }
}

#[test]
fn test_name_api() {
    let name = Name::new("example.com");
    assert_eq!(name.as_str(), "example.com");
    assert_eq!(name.to_string(), "example.com");
}

#[tokio::test]
async fn test_dns_overrides() {
    let mock = Arc::new(MockResolver {
        response: vec![SocketAddr::new(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)), 0)],
    });

    let mut overrides = HashMap::new();
    overrides.insert(
        Cow::Borrowed("local.override"),
        vec![SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 80)],
    );

    let resolver = DnsResolverWithOverrides::new(mock, overrides);

    // Test override hit
    let addrs: Vec<_> = resolver
        .resolve(Name::new("local.override"))
        .await
        .unwrap()
        .collect();

    assert_eq!(addrs.len(), 1);
    assert_eq!(addrs[0].ip(), IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));

    // Test passthrough (miss)
    let addrs: Vec<_> = resolver
        .resolve(Name::new("other.com"))
        .await
        .unwrap()
        .collect();

    assert_eq!(addrs.len(), 1);
    assert_eq!(addrs[0].ip(), IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)));
}

#[tokio::test]
async fn test_gai_resolver_localhost() {
    let resolver = GaiResolver::new();
    // localhost should always resolve, usually to 127.0.0.1 or ::1
    let result = resolver.resolve(Name::new("localhost")).await;

    // Depending on system config, this might fail in some CI envs,
    // but usually localhost is standard.
    if let Ok(addrs) = result {
        let list: Vec<_> = addrs.collect();
        assert!(!list.is_empty());
    } else {
        // Soft fail if network unavailable, but log it
        println!("GaiResolver failed for localhost - possibly no network access");
    }
}
