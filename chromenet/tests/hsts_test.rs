//! HSTS (HTTP Strict Transport Security) integration tests.

use chromenet::tls::hsts::HstsStore;

#[test]
fn test_preloaded_domains() {
    let store = HstsStore::with_preload();

    // Known preloaded domains should require upgrade
    assert!(store.should_upgrade("google.com"));
    assert!(store.should_upgrade("github.com"));
    assert!(store.should_upgrade("paypal.com"));
}

#[test]
fn test_subdomain_inheritance() {
    let store = HstsStore::with_preload();

    // Subdomains of preloaded domains should also upgrade
    assert!(store.should_upgrade("mail.google.com"));
    assert!(store.should_upgrade("api.github.com"));
    assert!(store.should_upgrade("www.paypal.com"));
}

#[test]
fn test_non_hsts_domains() {
    let store = HstsStore::with_preload();

    // Unknown domains should NOT be upgraded
    assert!(!store.should_upgrade("example.com"));
    assert!(!store.should_upgrade("localhost"));
    assert!(!store.should_upgrade("192.168.1.1"));
}

#[test]
fn test_dynamic_hsts_header() {
    let store = HstsStore::new();

    // Initially no upgrade required
    assert!(!store.should_upgrade("example.com"));

    // Simulate receiving HSTS header from server
    store.add_from_header("example.com", "max-age=31536000; includeSubDomains");

    // Now upgrade should be required
    assert!(store.should_upgrade("example.com"));
    assert!(store.should_upgrade("sub.example.com"));
}

#[test]
fn test_hsts_removal() {
    let store = HstsStore::new();

    // Add HSTS
    store.add_from_header("example.com", "max-age=31536000");
    assert!(store.should_upgrade("example.com"));

    // Remove via max-age=0
    store.add_from_header("example.com", "max-age=0");
    assert!(!store.should_upgrade("example.com"));
}

#[test]
fn test_case_insensitivity() {
    let store = HstsStore::new();
    store.add_preloaded("Example.COM", true);

    assert!(store.should_upgrade("example.com"));
    assert!(store.should_upgrade("EXAMPLE.COM"));
    assert!(store.should_upgrade("Example.Com"));
}
