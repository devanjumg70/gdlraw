//! Public Suffix List (PSL) integration tests.

use chromenet::cookies::psl::{is_public_suffix, is_valid_cookie_domain, registrable_domain};

#[test]
fn test_tld_is_public_suffix() {
    // Top-level domains are public suffixes
    assert!(is_public_suffix("com"));
    assert!(is_public_suffix("org"));
    assert!(is_public_suffix("net"));
    assert!(is_public_suffix("co.uk"));
    assert!(is_public_suffix("com.au"));
}

#[test]
fn test_domain_not_public_suffix() {
    // Normal domains are NOT public suffixes
    assert!(!is_public_suffix("example.com"));
    assert!(!is_public_suffix("google.com"));
    assert!(!is_public_suffix("bbc.co.uk"));
}

#[test]
fn test_registrable_domain_extraction() {
    // eTLD+1 extraction
    assert_eq!(
        registrable_domain("www.example.com"),
        Some("example.com".to_string())
    );
    assert_eq!(
        registrable_domain("sub.example.com"),
        Some("example.com".to_string())
    );
    assert_eq!(
        registrable_domain("www.bbc.co.uk"),
        Some("bbc.co.uk".to_string())
    );
}

#[test]
fn test_cookie_domain_validation() {
    // Valid: cookie domain matches request host (cookie_domain, url_host)
    assert!(is_valid_cookie_domain("example.com", "example.com"));

    // Valid: cookie domain is parent of request host
    // is_valid_cookie_domain(cookie_domain, url_host) - subdomain can use parent cookie
    assert!(is_valid_cookie_domain("example.com", "sub.example.com"));

    // Invalid: trying to set cookie on public suffix
    assert!(!is_valid_cookie_domain(".com", "example.com"));

    // Invalid: cookie domain doesn't match host
    assert!(!is_valid_cookie_domain("other.com", "example.com"));
}

#[test]
fn test_supercookie_prevention() {
    // These should all be REJECTED to prevent supercookie attacks
    assert!(!is_valid_cookie_domain("example.com", ".com"));
    assert!(!is_valid_cookie_domain("example.co.uk", ".co.uk"));
    assert!(!is_valid_cookie_domain("user.github.io", ".github.io"));
}

#[test]
fn test_wildcard_tlds() {
    // Some TLDs have wildcard rules
    assert!(is_public_suffix("github.io")); // github.io is a PSL entry
    assert!(!is_public_suffix("user.github.io")); // but user.github.io is not
}
