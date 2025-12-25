//! Certificate pinning for MITM prevention.
//!
//! Validates server certificates against expected SPKI (Subject Public Key Info)
//! SHA-256 hashes. Based on Chromium's TransportSecurityState.
//!
//! Note: HPKP (HTTP Public Key Pinning) is deprecated in browsers, but
//! preloaded pins and programmatic pinning are still valuable for security.

use crate::base::neterror::NetError;
use dashmap::DashMap;
use std::sync::Arc;
use time::OffsetDateTime;

/// SHA-256 hash of a certificate's SPKI (Subject Public Key Info).
pub type SpkiHash = [u8; 32];

/// A set of pins for a domain.
#[derive(Debug, Clone)]
pub struct PinSet {
    /// The domain this pin set applies to.
    pub domain: String,
    /// Whether to apply to subdomains.
    pub include_subdomains: bool,
    /// List of allowed SPKI SHA-256 hashes.
    pub pins: Vec<SpkiHash>,
    /// Optional expiration time (fail-open after expiry).
    pub expires: Option<OffsetDateTime>,
}

impl PinSet {
    /// Create a new pin set for a domain.
    pub fn new(domain: impl Into<String>) -> Self {
        Self {
            domain: domain.into(),
            include_subdomains: false,
            pins: Vec::new(),
            expires: None,
        }
    }

    /// Add a pin (base64-encoded SHA-256 hash).
    pub fn add_pin_base64(&mut self, pin_base64: &str) -> Result<(), NetError> {
        let decoded =
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, pin_base64)
                .map_err(|_| NetError::InvalidUrl)?;

        if decoded.len() != 32 {
            return Err(NetError::InvalidUrl);
        }

        let mut hash = [0u8; 32];
        hash.copy_from_slice(&decoded);
        self.pins.push(hash);
        Ok(())
    }

    /// Add a pin from raw bytes.
    pub fn add_pin(&mut self, hash: SpkiHash) {
        self.pins.push(hash);
    }

    /// Set include_subdomains flag.
    pub fn include_subdomains(mut self, include: bool) -> Self {
        self.include_subdomains = include;
        self
    }

    /// Set expiration time.
    pub fn expires_at(mut self, time: OffsetDateTime) -> Self {
        self.expires = Some(time);
        self
    }

    /// Check if pin set is expired.
    pub fn is_expired(&self) -> bool {
        if let Some(exp) = self.expires {
            OffsetDateTime::now_utc() > exp
        } else {
            false
        }
    }

    /// Check if any of the provided hashes match the pins.
    pub fn matches(&self, cert_hashes: &[SpkiHash]) -> bool {
        for hash in cert_hashes {
            if self.pins.contains(hash) {
                return true;
            }
        }
        false
    }
}

/// Thread-safe store for certificate pins.
#[derive(Clone)]
pub struct PinStore {
    pins: Arc<DashMap<String, PinSet>>,
}

impl Default for PinStore {
    fn default() -> Self {
        Self::new()
    }
}

impl PinStore {
    /// Create a new empty pin store.
    pub fn new() -> Self {
        Self {
            pins: Arc::new(DashMap::new()),
        }
    }

    /// Add or replace a pin set.
    pub fn add(&self, pin_set: PinSet) {
        self.pins.insert(pin_set.domain.to_lowercase(), pin_set);
    }

    /// Remove pins for a domain.
    pub fn remove(&self, domain: &str) {
        self.pins.remove(&domain.to_lowercase());
    }

    /// Check if the connection to `host` with given certificate hashes is allowed.
    /// Returns Ok(()) if pins match or no pins exist for this domain.
    /// Returns Err(CertPinningFailed) if pins exist but don't match.
    pub fn check(&self, host: &str, cert_hashes: &[SpkiHash]) -> Result<(), NetError> {
        let host_lower = host.to_lowercase();

        // Check for exact domain match
        if let Some(pin_set) = self.pins.get(&host_lower) {
            return self.verify_pins(&pin_set, cert_hashes);
        }

        // Check parent domains for wildcard pins
        let parts: Vec<&str> = host_lower.split('.').collect();
        for i in 1..parts.len() {
            let parent = parts[i..].join(".");
            if let Some(pin_set) = self.pins.get(&parent) {
                if pin_set.include_subdomains {
                    return self.verify_pins(&pin_set, cert_hashes);
                }
            }
        }

        // No pins configured for this host - allow
        Ok(())
    }

    fn verify_pins(&self, pin_set: &PinSet, cert_hashes: &[SpkiHash]) -> Result<(), NetError> {
        // Expired pins fail-open (like Chromium)
        if pin_set.is_expired() {
            return Ok(());
        }

        // Check if any cert in chain matches any pin
        if pin_set.matches(cert_hashes) {
            Ok(())
        } else {
            Err(NetError::CertPinningFailed)
        }
    }

    /// Get the number of pinned domains.
    pub fn len(&self) -> usize {
        self.pins.len()
    }

    /// Check if store is empty.
    pub fn is_empty(&self) -> bool {
        self.pins.is_empty()
    }
}

/// Compute SPKI hash from a DER-encoded certificate.
/// Returns SHA-256 hash of the Subject Public Key Info.
pub fn spki_hash(cert_der: &[u8]) -> Result<SpkiHash, NetError> {
    use boring::hash::{hash, MessageDigest};
    use boring::x509::X509;

    // Parse the certificate
    let cert = X509::from_der(cert_der).map_err(|_| NetError::CertPinningFailed)?;

    // Get the public key in DER format (this is the SPKI)
    let pubkey = cert.public_key().map_err(|_| NetError::CertPinningFailed)?;
    let spki_der = pubkey
        .public_key_to_der()
        .map_err(|_| NetError::CertPinningFailed)?;

    // Hash with SHA-256
    let digest =
        hash(MessageDigest::sha256(), &spki_der).map_err(|_| NetError::CertPinningFailed)?;

    let mut result = [0u8; 32];
    result.copy_from_slice(&digest);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pin_set_new() {
        let pin_set = PinSet::new("example.com");
        assert_eq!(pin_set.domain, "example.com");
        assert!(!pin_set.include_subdomains);
        assert!(pin_set.pins.is_empty());
    }

    #[test]
    fn test_pin_set_add_pin() {
        let mut pin_set = PinSet::new("example.com");
        let hash = [0u8; 32];
        pin_set.add_pin(hash);
        assert_eq!(pin_set.pins.len(), 1);
    }

    #[test]
    fn test_pin_set_matches() {
        let mut pin_set = PinSet::new("example.com");
        let hash1 = [1u8; 32];
        let hash2 = [2u8; 32];
        pin_set.add_pin(hash1);

        assert!(pin_set.matches(&[hash1]));
        assert!(!pin_set.matches(&[hash2]));
    }

    #[test]
    fn test_pin_store_no_pins() {
        let store = PinStore::new();
        let result = store.check("example.com", &[[0u8; 32]]);
        assert!(result.is_ok()); // No pins = allow
    }

    #[test]
    fn test_pin_store_matching_pin() {
        let store = PinStore::new();
        let mut pin_set = PinSet::new("example.com");
        let hash = [42u8; 32];
        pin_set.add_pin(hash);
        store.add(pin_set);

        let result = store.check("example.com", &[hash]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_pin_store_mismatched_pin() {
        let store = PinStore::new();
        let mut pin_set = PinSet::new("example.com");
        pin_set.add_pin([1u8; 32]);
        store.add(pin_set);

        let result = store.check("example.com", &[[2u8; 32]]);
        assert!(matches!(result, Err(NetError::CertPinningFailed)));
    }

    #[test]
    fn test_pin_store_subdomain() {
        let store = PinStore::new();
        let mut pin_set = PinSet::new("example.com").include_subdomains(true);
        let hash = [99u8; 32];
        pin_set.add_pin(hash);
        store.add(pin_set);

        // Subdomain should match
        let result = store.check("sub.example.com", &[hash]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_pin_store_expired_fails_open() {
        let store = PinStore::new();
        let mut pin_set = PinSet::new("example.com")
            .expires_at(OffsetDateTime::now_utc() - time::Duration::hours(1));
        pin_set.add_pin([1u8; 32]);
        store.add(pin_set);

        // Expired pin should fail-open (allow any cert)
        let result = store.check("example.com", &[[99u8; 32]]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_pin_store_case_insensitive() {
        let store = PinStore::new();
        let mut pin_set = PinSet::new("Example.COM");
        let hash = [77u8; 32];
        pin_set.add_pin(hash);
        store.add(pin_set);

        let result = store.check("example.com", &[hash]);
        assert!(result.is_ok());
    }
}
