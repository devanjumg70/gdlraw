//! HTTP Authentication Cache for proxy and server credentials.
//!
//! Caches authentication credentials to avoid re-prompting users.
//! Based on Chromium's HttpAuthCache.

use crate::http::digestauth::DigestAuthHandler;
use dashmap::DashMap;
use std::sync::Arc;

/// Authentication scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthScheme {
    /// Basic authentication (base64 encoded)
    Basic,
    /// Digest authentication (challenge-response, RFC 7616)
    Digest,
}

/// Cached authentication entry for Basic auth.
#[derive(Debug, Clone)]
pub struct BasicAuthEntry {
    /// Realm from WWW-Authenticate header
    pub realm: String,
    /// Username
    pub username: String,
    /// Password
    pub password: String,
}

impl BasicAuthEntry {
    /// Create a new basic auth entry.
    pub fn new(
        realm: impl Into<String>,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        Self {
            realm: realm.into(),
            username: username.into(),
            password: password.into(),
        }
    }

    /// Generate the Authorization header value.
    pub fn to_header_value(&self) -> String {
        use base64::{engine::general_purpose, Engine as _};
        let creds = format!("{}:{}", self.username, self.password);
        let encoded = general_purpose::STANDARD.encode(creds);
        format!("Basic {}", encoded)
    }
}

/// Cached Digest authentication session.
#[derive(Debug, Clone)]
pub struct DigestAuthSession {
    /// The Digest handler with parsed challenge and nonce count
    pub handler: DigestAuthHandler,
    /// Username for this session
    pub username: String,
    /// Password for this session (consider using Zeroizing in production)
    pub password: String,
}

impl DigestAuthSession {
    /// Create a new Digest auth session from a parsed challenge.
    pub fn new(
        handler: DigestAuthHandler,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        Self {
            handler,
            username: username.into(),
            password: password.into(),
        }
    }

    /// Generate the next Authorization header value.
    ///
    /// Increments the nonce count automatically.
    pub fn generate_auth_header(&mut self, method: &str, uri: &str) -> String {
        self.handler
            .generate_auth_token(method, uri, &self.username, &self.password)
    }
}

/// Thread-safe authentication cache.
/// Keys entries by host:port + realm.
#[derive(Clone)]
pub struct AuthCache {
    basic_entries: Arc<DashMap<String, BasicAuthEntry>>,
    digest_sessions: Arc<DashMap<String, DigestAuthSession>>,
}

impl Default for AuthCache {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthCache {
    /// Create a new empty auth cache.
    pub fn new() -> Self {
        Self {
            basic_entries: Arc::new(DashMap::new()),
            digest_sessions: Arc::new(DashMap::new()),
        }
    }

    /// Generate cache key from host, port, and realm.
    fn key(host: &str, port: u16, realm: &str) -> String {
        format!("{}:{}:{}", host.to_lowercase(), port, realm)
    }

    // --- Basic Auth Methods ---

    /// Lookup cached Basic credentials for a host and realm.
    pub fn lookup_basic(&self, host: &str, port: u16, realm: &str) -> Option<BasicAuthEntry> {
        let key = Self::key(host, port, realm);
        self.basic_entries.get(&key).map(|e| e.clone())
    }

    /// Store Basic credentials for a host and realm.
    pub fn store_basic(&self, host: &str, port: u16, realm: &str, entry: BasicAuthEntry) {
        let key = Self::key(host, port, realm);
        self.basic_entries.insert(key, entry);
    }

    // --- Digest Auth Methods ---

    /// Lookup cached Digest session for a host and realm.
    pub fn lookup_digest(&self, host: &str, port: u16, realm: &str) -> Option<DigestAuthSession> {
        let key = Self::key(host, port, realm);
        self.digest_sessions.get(&key).map(|e| e.clone())
    }

    /// Store a Digest session for a host and realm.
    pub fn store_digest(&self, host: &str, port: u16, realm: &str, session: DigestAuthSession) {
        let key = Self::key(host, port, realm);
        self.digest_sessions.insert(key, session);
    }

    /// Generate Authorization header for Digest auth.
    ///
    /// Looks up the cached session and generates the next auth token.
    /// Returns None if no session is cached for this realm.
    pub fn generate_digest_header(
        &self,
        host: &str,
        port: u16,
        realm: &str,
        method: &str,
        uri: &str,
    ) -> Option<String> {
        let key = Self::key(host, port, realm);
        self.digest_sessions
            .get_mut(&key)
            .map(|mut session| session.generate_auth_header(method, uri))
    }

    // --- General Methods ---

    /// Remove all credentials for a host (all realms).
    pub fn remove_host(&self, host: &str, port: u16) {
        let prefix = format!("{}:{}", host.to_lowercase(), port);
        self.basic_entries.retain(|k, _| !k.starts_with(&prefix));
        self.digest_sessions.retain(|k, _| !k.starts_with(&prefix));
    }

    /// Clear all cached credentials.
    pub fn clear(&self) {
        self.basic_entries.clear();
        self.digest_sessions.clear();
    }

    /// Get total number of cached entries.
    pub fn len(&self) -> usize {
        self.basic_entries.len() + self.digest_sessions.len()
    }

    /// Check if cache is empty.
    pub fn is_empty(&self) -> bool {
        self.basic_entries.is_empty() && self.digest_sessions.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_auth_store_and_lookup() {
        let cache = AuthCache::new();
        let entry = BasicAuthEntry::new("MyRealm", "user", "pass");

        cache.store_basic("proxy.example.com", 8080, "MyRealm", entry);

        let found = cache.lookup_basic("proxy.example.com", 8080, "MyRealm");
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.username, "user");
        assert_eq!(found.password, "pass");
    }

    #[test]
    fn test_basic_auth_lookup_not_found() {
        let cache = AuthCache::new();
        assert!(cache.lookup_basic("unknown.com", 80, "Realm").is_none());
    }

    #[test]
    fn test_basic_auth_different_realms() {
        let cache = AuthCache::new();
        cache.store_basic(
            "proxy.com",
            80,
            "Realm1",
            BasicAuthEntry::new("Realm1", "user1", "pass1"),
        );
        cache.store_basic(
            "proxy.com",
            80,
            "Realm2",
            BasicAuthEntry::new("Realm2", "user2", "pass2"),
        );

        let r1 = cache.lookup_basic("proxy.com", 80, "Realm1").unwrap();
        let r2 = cache.lookup_basic("proxy.com", 80, "Realm2").unwrap();

        assert_eq!(r1.username, "user1");
        assert_eq!(r2.username, "user2");
    }

    #[test]
    fn test_remove_host() {
        let cache = AuthCache::new();
        cache.store_basic(
            "proxy.com",
            80,
            "Realm1",
            BasicAuthEntry::new("Realm1", "u", "p"),
        );
        cache.store_basic(
            "proxy.com",
            80,
            "Realm2",
            BasicAuthEntry::new("Realm2", "u", "p"),
        );
        cache.store_basic(
            "other.com",
            80,
            "Realm1",
            BasicAuthEntry::new("Realm1", "u", "p"),
        );

        cache.remove_host("proxy.com", 80);

        assert!(cache.lookup_basic("proxy.com", 80, "Realm1").is_none());
        assert!(cache.lookup_basic("proxy.com", 80, "Realm2").is_none());
        assert!(cache.lookup_basic("other.com", 80, "Realm1").is_some());
    }

    #[test]
    fn test_case_insensitive_host() {
        let cache = AuthCache::new();
        cache.store_basic(
            "Proxy.COM",
            80,
            "Realm",
            BasicAuthEntry::new("Realm", "u", "p"),
        );

        assert!(cache.lookup_basic("proxy.com", 80, "Realm").is_some());
        assert!(cache.lookup_basic("PROXY.COM", 80, "Realm").is_some());
    }

    #[test]
    fn test_to_header_value() {
        let entry = BasicAuthEntry::new("Realm", "user", "pass");
        let header = entry.to_header_value();

        // base64("user:pass") = "dXNlcjpwYXNz"
        assert_eq!(header, "Basic dXNlcjpwYXNz");
    }

    #[test]
    fn test_clear() {
        let cache = AuthCache::new();
        cache.store_basic("a.com", 80, "R", BasicAuthEntry::new("R", "u", "p"));
        cache.store_basic("b.com", 80, "R", BasicAuthEntry::new("R", "u", "p"));

        cache.clear();

        assert!(cache.is_empty());
    }

    #[test]
    fn test_digest_auth_session() {
        let cache = AuthCache::new();
        let challenge = r#"realm="test", nonce="abc123", qop="auth""#;
        let handler = DigestAuthHandler::parse_challenge(challenge).unwrap();

        let session = DigestAuthSession::new(handler, "user", "pass");
        cache.store_digest("example.com", 80, "test", session);

        // Generate header through cache
        let header = cache.generate_digest_header("example.com", 80, "test", "GET", "/api");
        assert!(header.is_some());
        assert!(header.unwrap().starts_with("Digest username=\"user\""));
    }
}
