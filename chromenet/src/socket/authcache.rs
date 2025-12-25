//! HTTP Authentication Cache for proxy and server credentials.
//!
//! Caches authentication credentials to avoid re-prompting users.
//! Based on Chromium's HttpAuthCache.

use dashmap::DashMap;
use std::sync::Arc;

/// Authentication scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthScheme {
    /// Basic authentication (base64 encoded)
    Basic,
    /// Digest authentication (challenge-response)
    Digest,
    /// NTLM/Negotiate (Windows integrated auth)
    Ntlm,
}

/// Cached authentication entry.
#[derive(Debug, Clone)]
pub struct AuthEntry {
    /// Authentication scheme
    pub scheme: AuthScheme,
    /// Realm from WWW-Authenticate or Proxy-Authenticate header
    pub realm: String,
    /// Username
    pub username: String,
    /// Password
    pub password: String,
}

impl AuthEntry {
    /// Create a new basic auth entry.
    pub fn basic(
        realm: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        Self {
            scheme: AuthScheme::Basic,
            realm: realm.into(),
            username: username.into(),
            password: password.into(),
        }
    }

    /// Generate the Authorization header value.
    pub fn to_header_value(&self) -> String {
        match self.scheme {
            AuthScheme::Basic => {
                use base64::{engine::general_purpose, Engine as _};
                let creds = format!("{}:{}", self.username, self.password);
                let encoded = general_purpose::STANDARD.encode(creds);
                format!("Basic {}", encoded)
            }
            AuthScheme::Digest => {
                // Digest auth requires challenge params - simplified stub
                format!("Digest username=\"{}\"", self.username)
            }
            AuthScheme::Ntlm => {
                // NTLM is complex multi-step - stub
                "NTLM".to_string()
            }
        }
    }
}

/// Thread-safe authentication cache.
/// Keys entries by host:port + realm.
#[derive(Clone)]
pub struct AuthCache {
    entries: Arc<DashMap<String, AuthEntry>>,
}

impl Default for AuthCache {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthCache {
    /// Create a new empty auth cache.
    pub fn new() -> Self {
        Self { entries: Arc::new(DashMap::new()) }
    }

    /// Generate cache key from host and realm.
    fn key(host: &str, port: u16, realm: &str) -> String {
        format!("{}:{}:{}", host.to_lowercase(), port, realm)
    }

    /// Lookup cached credentials for a host and realm.
    pub fn lookup(&self, host: &str, port: u16, realm: &str) -> Option<AuthEntry> {
        let key = Self::key(host, port, realm);
        self.entries.get(&key).map(|e| e.clone())
    }

    /// Store credentials for a host and realm.
    pub fn store(&self, host: &str, port: u16, realm: &str, entry: AuthEntry) {
        let key = Self::key(host, port, realm);
        self.entries.insert(key, entry);
    }

    /// Remove credentials for a host (all realms).
    pub fn remove_host(&self, host: &str, port: u16) {
        let prefix = format!("{}:{}", host.to_lowercase(), port);
        self.entries.retain(|k, _| !k.starts_with(&prefix));
    }

    /// Clear all cached credentials.
    pub fn clear(&self) {
        self.entries.clear();
    }

    /// Get number of cached entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if cache is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_and_lookup() {
        let cache = AuthCache::new();
        let entry = AuthEntry::basic("MyRealm", "user", "pass");

        cache.store("proxy.example.com", 8080, "MyRealm", entry);

        let found = cache.lookup("proxy.example.com", 8080, "MyRealm");
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.username, "user");
        assert_eq!(found.password, "pass");
    }

    #[test]
    fn test_lookup_not_found() {
        let cache = AuthCache::new();
        assert!(cache.lookup("unknown.com", 80, "Realm").is_none());
    }

    #[test]
    fn test_different_realms() {
        let cache = AuthCache::new();
        cache.store("proxy.com", 80, "Realm1", AuthEntry::basic("Realm1", "user1", "pass1"));
        cache.store("proxy.com", 80, "Realm2", AuthEntry::basic("Realm2", "user2", "pass2"));

        let r1 = cache.lookup("proxy.com", 80, "Realm1").unwrap();
        let r2 = cache.lookup("proxy.com", 80, "Realm2").unwrap();

        assert_eq!(r1.username, "user1");
        assert_eq!(r2.username, "user2");
    }

    #[test]
    fn test_remove_host() {
        let cache = AuthCache::new();
        cache.store("proxy.com", 80, "Realm1", AuthEntry::basic("Realm1", "u", "p"));
        cache.store("proxy.com", 80, "Realm2", AuthEntry::basic("Realm2", "u", "p"));
        cache.store("other.com", 80, "Realm1", AuthEntry::basic("Realm1", "u", "p"));

        cache.remove_host("proxy.com", 80);

        assert!(cache.lookup("proxy.com", 80, "Realm1").is_none());
        assert!(cache.lookup("proxy.com", 80, "Realm2").is_none());
        assert!(cache.lookup("other.com", 80, "Realm1").is_some());
    }

    #[test]
    fn test_case_insensitive_host() {
        let cache = AuthCache::new();
        cache.store("Proxy.COM", 80, "Realm", AuthEntry::basic("Realm", "u", "p"));

        assert!(cache.lookup("proxy.com", 80, "Realm").is_some());
        assert!(cache.lookup("PROXY.COM", 80, "Realm").is_some());
    }

    #[test]
    fn test_to_header_value() {
        let entry = AuthEntry::basic("Realm", "user", "pass");
        let header = entry.to_header_value();

        // base64("user:pass") = "dXNlcjpwYXNz"
        assert_eq!(header, "Basic dXNlcjpwYXNz");
    }

    #[test]
    fn test_clear() {
        let cache = AuthCache::new();
        cache.store("a.com", 80, "R", AuthEntry::basic("R", "u", "p"));
        cache.store("b.com", 80, "R", AuthEntry::basic("R", "u", "p"));

        cache.clear();

        assert!(cache.is_empty());
    }
}
