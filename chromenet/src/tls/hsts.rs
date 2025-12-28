//! HTTP Strict Transport Security (HSTS) implementation.
//!
//! Enforces HTTPS for domains that require it, supporting both:
//! - Static preload list (hardcoded domains)
//! - Dynamic HSTS headers from Strict-Transport-Security
//!
//! Based on Chromium's TransportSecurityState.

use dashmap::DashMap;
use std::sync::Arc;
use time::{Duration, OffsetDateTime};

/// HSTS entry for a domain.
#[derive(Debug, Clone)]
pub struct HstsEntry {
    /// Whether subdomains should also be upgraded.
    pub include_subdomains: bool,
    /// When this entry expires (None = permanent/preloaded).
    pub expires: Option<OffsetDateTime>,
}

impl HstsEntry {
    /// Create a new HSTS entry.
    pub fn new(include_subdomains: bool, max_age_secs: Option<u64>) -> Self {
        let expires =
            max_age_secs.map(|secs| OffsetDateTime::now_utc() + Duration::seconds(secs as i64));
        Self {
            include_subdomains,
            expires,
        }
    }

    /// Create a permanent/preloaded entry.
    pub fn preloaded(include_subdomains: bool) -> Self {
        Self {
            include_subdomains,
            expires: None,
        }
    }

    /// Check if this entry has expired.
    pub fn is_expired(&self) -> bool {
        if let Some(exp) = self.expires {
            OffsetDateTime::now_utc() > exp
        } else {
            false
        }
    }
}

/// Thread-safe HSTS store.
#[derive(Clone)]
pub struct HstsStore {
    entries: Arc<DashMap<String, HstsEntry>>,
}

impl Default for HstsStore {
    fn default() -> Self {
        Self::new()
    }
}

impl HstsStore {
    /// Create a new empty HSTS store.
    pub fn new() -> Self {
        Self {
            entries: Arc::new(DashMap::new()),
        }
    }

    /// Create an HSTS store with common preloaded domains.
    pub fn with_preload() -> Self {
        let store = Self::new();

        // Common preloaded HSTS domains (subset of Chromium's list)
        let preloaded = [
            ("google.com", true),
            ("accounts.google.com", true),
            ("mail.google.com", true),
            ("youtube.com", true),
            ("facebook.com", true),
            ("twitter.com", true),
            ("github.com", true),
            ("paypal.com", true),
            ("stripe.com", true),
            ("cloudflare.com", true),
        ];

        for (domain, include_subdomains) in preloaded {
            store.add_preloaded(domain, include_subdomains);
        }

        store
    }

    /// Add a preloaded (permanent) HSTS entry.
    pub fn add_preloaded(&self, domain: &str, include_subdomains: bool) {
        self.entries.insert(
            domain.to_lowercase(),
            HstsEntry::preloaded(include_subdomains),
        );
    }

    /// Check if a host should be upgraded to HTTPS.
    ///
    /// Chromium: net/http/transport_security_state.cc
    pub fn should_upgrade(&self, host: &str) -> bool {
        let host_lower = host.to_lowercase();

        // Check exact match
        if let Some(entry) = self.entries.get(&host_lower) {
            if !entry.is_expired() {
                return true;
            }
        }

        // Check parent domains for include_subdomains
        // Optimization: Zero-allocation iteration over parent domains
        let mut current = host_lower.as_str();
        while let Some(idx) = current.find('.') {
            if idx + 1 >= current.len() {
                break;
            }
            current = &current[idx + 1..];
            if let Some(entry) = self.entries.get(current) {
                if !entry.is_expired() && entry.include_subdomains {
                    return true;
                }
            }
        }

        false
    }

    /// Parse and add HSTS from a Strict-Transport-Security header.
    /// Format: "max-age=31536000; includeSubDomains; preload"
    pub fn add_from_header(&self, host: &str, header: &str) {
        let mut max_age: Option<u64> = None;
        let mut include_subdomains = false;

        for part in header.split(';') {
            let part = part.trim().to_lowercase();

            if let Some(age_str) = part.strip_prefix("max-age=") {
                if let Ok(secs) = age_str.parse::<u64>() {
                    max_age = Some(secs);
                }
            } else if part == "includesubdomains" {
                include_subdomains = true;
            }
            // "preload" directive is informational only
        }

        if let Some(secs) = max_age {
            if secs == 0 {
                // max-age=0 removes the entry
                self.entries.remove(&host.to_lowercase());
            } else {
                self.entries.insert(
                    host.to_lowercase(),
                    HstsEntry::new(include_subdomains, Some(secs)),
                );
            }
        }
    }

    /// Get the number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if store is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Save HSTS entries to a JSON file.
    ///
    /// Serializes non-expired entries for persistence across restarts.
    pub fn save_to_file(&self, path: &std::path::Path) -> std::io::Result<()> {
        use std::io::Write;

        #[derive(serde::Serialize)]
        struct SerializableEntry {
            domain: String,
            include_subdomains: bool,
            expires_timestamp: Option<i64>,
        }

        let entries: Vec<SerializableEntry> = self
            .entries
            .iter()
            .filter(|e| !e.is_expired())
            .map(|e| SerializableEntry {
                domain: e.key().clone(),
                include_subdomains: e.include_subdomains,
                expires_timestamp: e.expires.map(|dt| dt.unix_timestamp()),
            })
            .collect();

        let json = serde_json::to_string_pretty(&entries)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let mut file = std::fs::File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    /// Load HSTS entries from a JSON file.
    ///
    /// Restores entries from a previous save. Expired entries are skipped.
    pub fn load_from_file(&self, path: &std::path::Path) -> std::io::Result<usize> {
        use std::io::Read;

        #[derive(serde::Deserialize)]
        struct SerializableEntry {
            domain: String,
            include_subdomains: bool,
            expires_timestamp: Option<i64>,
        }

        let mut file = std::fs::File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let entries: Vec<SerializableEntry> = serde_json::from_str(&contents)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let mut loaded = 0;
        for entry in entries {
            let expires = entry
                .expires_timestamp
                .and_then(|ts| OffsetDateTime::from_unix_timestamp(ts).ok());

            // Skip expired entries
            if let Some(exp) = expires {
                if OffsetDateTime::now_utc() > exp {
                    continue;
                }
            }

            self.entries.insert(
                entry.domain,
                HstsEntry {
                    include_subdomains: entry.include_subdomains,
                    expires,
                },
            );
            loaded += 1;
        }

        Ok(loaded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_upgrade_exact_match() {
        let store = HstsStore::new();
        store.add_preloaded("example.com", false);

        assert!(store.should_upgrade("example.com"));
        assert!(store.should_upgrade("EXAMPLE.COM"));
        assert!(!store.should_upgrade("sub.example.com"));
    }

    #[test]
    fn test_should_upgrade_subdomain() {
        let store = HstsStore::new();
        store.add_preloaded("example.com", true);

        assert!(store.should_upgrade("example.com"));
        assert!(store.should_upgrade("sub.example.com"));
        assert!(store.should_upgrade("deep.sub.example.com"));
    }

    #[test]
    fn test_no_upgrade_for_unknown() {
        let store = HstsStore::new();
        assert!(!store.should_upgrade("unknown.com"));
    }

    #[test]
    fn test_add_from_header() {
        let store = HstsStore::new();
        store.add_from_header("example.com", "max-age=31536000; includeSubDomains");

        assert!(store.should_upgrade("example.com"));
        assert!(store.should_upgrade("sub.example.com"));
    }

    #[test]
    fn test_add_from_header_no_subdomains() {
        let store = HstsStore::new();
        store.add_from_header("example.com", "max-age=31536000");

        assert!(store.should_upgrade("example.com"));
        assert!(!store.should_upgrade("sub.example.com"));
    }

    #[test]
    fn test_max_age_zero_removes() {
        let store = HstsStore::new();
        store.add_preloaded("example.com", true);
        store.add_from_header("example.com", "max-age=0");

        assert!(!store.should_upgrade("example.com"));
    }

    #[test]
    fn test_with_preload() {
        let store = HstsStore::with_preload();

        assert!(store.should_upgrade("google.com"));
        assert!(store.should_upgrade("github.com"));
        assert!(store.should_upgrade("mail.google.com")); // subdomain
    }

    #[test]
    fn test_case_insensitive() {
        let store = HstsStore::new();
        store.add_preloaded("Example.COM", true);

        assert!(store.should_upgrade("example.com"));
        assert!(store.should_upgrade("EXAMPLE.COM"));
    }
}
