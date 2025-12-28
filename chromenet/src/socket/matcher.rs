//! Proxy matching with NO_PROXY support.
//!
//! Provides domain and IP matching for proxy bypass rules.

use std::net::IpAddr;
use url::Url;

/// Proxy matcher with NO_PROXY rules.
///
/// Based on curl's NO_PROXY behavior:
/// - Entries are comma-separated
/// - IP addresses and CIDR ranges supported
/// - Domain matching with optional leading dot
/// - Wildcard `*` matches all hosts
#[derive(Debug, Clone, Default)]
pub struct ProxyMatcher {
    domains: Vec<String>,
    ips: Vec<IpMatch>,
    match_all: bool,
}

#[derive(Debug, Clone)]
enum IpMatch {
    Address(IpAddr),
    Cidr(IpAddr, u8),
}

impl ProxyMatcher {
    /// Create from environment variables.
    ///
    /// Checks `NO_PROXY` then `no_proxy`.
    pub fn from_env() -> Self {
        let raw = std::env::var("NO_PROXY")
            .or_else(|_| std::env::var("no_proxy"))
            .unwrap_or_default();
        Self::from_string(&raw)
    }

    /// Create from a NO_PROXY string.
    ///
    /// Format: comma-separated list of:
    /// - Domain names (with optional leading dot): `.example.com`, `example.com`
    /// - IP addresses: `192.168.1.1`
    /// - CIDR ranges: `192.168.1.0/24`
    /// - Wildcard: `*` matches everything
    pub fn from_string(no_proxy: &str) -> Self {
        let mut matcher = ProxyMatcher::default();

        for part in no_proxy.split(',').map(str::trim) {
            if part.is_empty() {
                continue;
            }

            // Wildcard matches all
            if part == "*" {
                matcher.match_all = true;
                continue;
            }

            // Try to parse as CIDR
            if let Some((ip_str, prefix_str)) = part.split_once('/') {
                if let Ok(ip) = ip_str.parse::<IpAddr>() {
                    if let Ok(prefix) = prefix_str.parse::<u8>() {
                        matcher.ips.push(IpMatch::Cidr(ip, prefix));
                        continue;
                    }
                }
            }

            // Try to parse as IP
            if let Ok(ip) = part.parse::<IpAddr>() {
                matcher.ips.push(IpMatch::Address(ip));
                continue;
            }

            // Otherwise treat as domain
            matcher.domains.push(part.to_lowercase());
        }

        matcher
    }

    /// Check if a host should bypass the proxy.
    pub fn should_bypass(&self, host: &str) -> bool {
        if self.match_all {
            return true;
        }

        // Strip brackets from IPv6
        let host = host.trim_start_matches('[').trim_end_matches(']');

        // Try as IP first
        if let Ok(ip) = host.parse::<IpAddr>() {
            return self.ip_matches(ip);
        }

        // Check domain matching
        self.domain_matches(host)
    }

    /// Check if URL should bypass proxy.
    pub fn should_bypass_url(&self, url: &Url) -> bool {
        url.host_str().is_some_and(|h| self.should_bypass(h))
    }

    fn ip_matches(&self, addr: IpAddr) -> bool {
        for ip_match in &self.ips {
            match ip_match {
                IpMatch::Address(ip) => {
                    if &addr == ip {
                        return true;
                    }
                }
                IpMatch::Cidr(network, prefix) => {
                    if cidr_contains(*network, *prefix, addr) {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn domain_matches(&self, host: &str) -> bool {
        let host_lower = host.to_lowercase();

        for domain in &self.domains {
            // Strip leading dot for matching
            let domain_clean = domain.strip_prefix('.').unwrap_or(domain);

            // Exact match
            if host_lower == domain_clean {
                return true;
            }

            // Subdomain match: host must end with ".domain"
            // e.g., "www.example.com" matches ".example.com" or "example.com"
            // but "notexample.com" should NOT match "example.com"
            let with_dot = format!(".{}", domain_clean);
            if host_lower.ends_with(&with_dot) {
                return true;
            }
        }
        false
    }
}

/// Check if IP is within CIDR range.
fn cidr_contains(network: IpAddr, prefix: u8, addr: IpAddr) -> bool {
    match (network, addr) {
        (IpAddr::V4(net), IpAddr::V4(ip)) => {
            if prefix > 32 {
                return false;
            }
            let mask = if prefix == 0 {
                0u32
            } else {
                !0u32 << (32 - prefix)
            };
            (u32::from(net) & mask) == (u32::from(ip) & mask)
        }
        (IpAddr::V6(net), IpAddr::V6(ip)) => {
            if prefix > 128 {
                return false;
            }
            let net_bits = u128::from(net);
            let ip_bits = u128::from(ip);
            let mask = if prefix == 0 {
                0u128
            } else {
                !0u128 << (128 - prefix)
            };
            (net_bits & mask) == (ip_bits & mask)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wildcard() {
        let m = ProxyMatcher::from_string("*");
        assert!(m.should_bypass("anything.com"));
        assert!(m.should_bypass("192.168.1.1"));
    }

    #[test]
    fn test_domain_exact() {
        let m = ProxyMatcher::from_string("example.com");
        assert!(m.should_bypass("example.com"));
        assert!(m.should_bypass("EXAMPLE.COM"));
        assert!(!m.should_bypass("notexample.com"));
    }

    #[test]
    fn test_domain_subdomain() {
        let m = ProxyMatcher::from_string(".example.com");
        assert!(m.should_bypass("example.com"));
        assert!(m.should_bypass("www.example.com"));
        assert!(m.should_bypass("sub.www.example.com"));
        assert!(!m.should_bypass("notexample.com"));
    }

    #[test]
    fn test_ip_exact() {
        let m = ProxyMatcher::from_string("192.168.1.1, 10.0.0.5");
        assert!(m.should_bypass("192.168.1.1"));
        assert!(m.should_bypass("10.0.0.5"));
        assert!(!m.should_bypass("192.168.1.2"));
    }

    #[test]
    fn test_cidr() {
        let m = ProxyMatcher::from_string("192.168.1.0/24");
        assert!(m.should_bypass("192.168.1.1"));
        assert!(m.should_bypass("192.168.1.254"));
        assert!(!m.should_bypass("192.168.2.1"));
    }

    #[test]
    fn test_ipv6() {
        let m = ProxyMatcher::from_string("::1, 2001:db8::/32");
        assert!(m.should_bypass("[::1]"));
        assert!(m.should_bypass("2001:db8::1"));
        assert!(!m.should_bypass("2001:db9::1"));
    }

    #[test]
    fn test_mixed() {
        let m = ProxyMatcher::from_string("localhost, .internal.company.com, 10.0.0.0/8");
        assert!(m.should_bypass("localhost"));
        assert!(m.should_bypass("api.internal.company.com"));
        assert!(m.should_bypass("10.1.2.3"));
        assert!(!m.should_bypass("external.com"));
    }
}
