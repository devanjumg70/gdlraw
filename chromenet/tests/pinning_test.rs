//! Certificate pinning integration tests.

use chromenet::tls::pinning::{PinSet, PinStore};

#[test]
fn test_pin_store_empty_allows_all() {
    let store = PinStore::new();

    // No pins configured = allow all connections
    let fake_hash = [0u8; 32];
    assert!(store.check("example.com", &[fake_hash]).is_ok());
}

#[test]
fn test_pin_match_allows_connection() {
    let store = PinStore::new();

    let expected_hash = [42u8; 32];
    let mut pin_set = PinSet::new("example.com");
    pin_set.add_pin(expected_hash);
    store.add(pin_set);

    // Matching pin should allow connection
    assert!(store.check("example.com", &[expected_hash]).is_ok());
}

#[test]
fn test_pin_mismatch_blocks_connection() {
    let store = PinStore::new();

    let expected_hash = [42u8; 32];
    let wrong_hash = [99u8; 32];

    let mut pin_set = PinSet::new("example.com");
    pin_set.add_pin(expected_hash);
    store.add(pin_set);

    // Wrong pin should block connection
    let result = store.check("example.com", &[wrong_hash]);
    assert!(result.is_err());
}

#[test]
fn test_subdomain_pinning() {
    let store = PinStore::new();

    let hash = [77u8; 32];
    let mut pin_set = PinSet::new("example.com").include_subdomains(true);
    pin_set.add_pin(hash);
    store.add(pin_set);

    // Subdomain should also be pinned
    assert!(store.check("sub.example.com", &[hash]).is_ok());
    assert!(store.check("deep.sub.example.com", &[hash]).is_ok());
}

#[test]
fn test_expired_pins_fail_open() {
    use time::OffsetDateTime;

    let store = PinStore::new();

    let hash = [1u8; 32];
    let wrong_hash = [2u8; 32];

    // Create expired pin set
    let mut pin_set =
        PinSet::new("example.com").expires_at(OffsetDateTime::now_utc() - time::Duration::hours(1));
    pin_set.add_pin(hash);
    store.add(pin_set);

    // Expired pins should fail-open (allow any cert)
    assert!(store.check("example.com", &[wrong_hash]).is_ok());
}

#[test]
fn test_multiple_pins_any_match() {
    let store = PinStore::new();

    let hash1 = [1u8; 32];
    let hash2 = [2u8; 32];
    let hash3 = [3u8; 32];

    let mut pin_set = PinSet::new("example.com");
    pin_set.add_pin(hash1);
    pin_set.add_pin(hash2);
    store.add(pin_set);

    // Any matching pin should work
    assert!(store.check("example.com", &[hash1]).is_ok());
    assert!(store.check("example.com", &[hash2]).is_ok());
    assert!(store.check("example.com", &[hash3]).is_err()); // Not in pin set
}
