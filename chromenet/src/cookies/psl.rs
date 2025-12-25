//! Public Suffix List (PSL) validation for cookie domain security.
//!
//! Prevents supercookie attacks by rejecting cookies set on public
//! suffixes like `.com`, `.co.uk`, etc.
//!
//! Uses Mozilla's Public Suffix List via the `psl` crate.

use psl::{List, Psl};

/// Check if a domain is a public suffix (e.g., "com", "co.uk").
/// Returns true if the domain itself is a public suffix.
pub fn is_public_suffix(domain: &str) -> bool {
    let domain_lower = domain.to_lowercase();
    let domain_bytes = domain_lower.as_bytes();

    // Get the suffix for this domain
    if let Some(suffix) = List.suffix(domain_bytes) {
        // The domain is a public suffix if it equals its own suffix
        suffix.as_bytes() == domain_bytes
    } else {
        // Unknown TLD - treat as potentially unsafe
        false
    }
}

/// Get the registrable domain (eTLD+1) for a domain.
/// For "sub.example.com", returns "example.com".
/// For "example.com", returns "example.com".
/// For "com" (public suffix), returns None.
pub fn registrable_domain(domain: &str) -> Option<String> {
    let domain_lower = domain.to_lowercase();
    psl::domain(domain_lower.as_bytes())
        .and_then(|d| std::str::from_utf8(d.as_bytes()).ok())
        .map(|s| s.to_string())
}

/// Check if a cookie domain is valid for a given URL.
/// The cookie domain must be a suffix of the URL's host and
/// must not be a public suffix.
pub fn is_valid_cookie_domain(cookie_domain: &str, url_host: &str) -> bool {
    // Remove leading dot from cookie domain if present
    let cookie_domain = cookie_domain.strip_prefix('.').unwrap_or(cookie_domain);
    let cookie_domain_lower = cookie_domain.to_lowercase();
    let url_host_lower = url_host.to_lowercase();

    // 1. Cookie domain must not be a public suffix
    if is_public_suffix(&cookie_domain_lower) {
        return false;
    }

    // 2. URL host must match or be a subdomain of cookie domain
    if url_host_lower == cookie_domain_lower {
        return true;
    }

    // Check if url_host ends with .cookie_domain
    if url_host_lower.ends_with(&format!(".{}", cookie_domain_lower)) {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_public_suffix_com() {
        assert!(is_public_suffix("com"));
        assert!(is_public_suffix("COM"));
    }

    #[test]
    fn test_is_public_suffix_co_uk() {
        assert!(is_public_suffix("co.uk"));
        assert!(is_public_suffix("CO.UK"));
    }

    #[test]
    fn test_is_public_suffix_github_io() {
        assert!(is_public_suffix("github.io"));
    }

    #[test]
    fn test_not_public_suffix() {
        assert!(!is_public_suffix("example.com"));
        assert!(!is_public_suffix("google.com"));
        assert!(!is_public_suffix("sub.example.com"));
    }

    #[test]
    fn test_registrable_domain() {
        assert_eq!(
            registrable_domain("example.com"),
            Some("example.com".to_string())
        );
        assert_eq!(
            registrable_domain("sub.example.com"),
            Some("example.com".to_string())
        );
        assert_eq!(
            registrable_domain("deep.sub.example.com"),
            Some("example.com".to_string())
        );
    }

    #[test]
    fn test_registrable_domain_co_uk() {
        assert_eq!(
            registrable_domain("example.co.uk"),
            Some("example.co.uk".to_string())
        );
        assert_eq!(
            registrable_domain("sub.example.co.uk"),
            Some("example.co.uk".to_string())
        );
    }

    #[test]
    fn test_registrable_domain_public_suffix() {
        // Public suffix has no registrable domain
        assert_eq!(registrable_domain("com"), None);
        assert_eq!(registrable_domain("co.uk"), None);
    }

    #[test]
    fn test_valid_cookie_domain() {
        assert!(is_valid_cookie_domain("example.com", "example.com"));
        assert!(is_valid_cookie_domain("example.com", "sub.example.com"));
        assert!(is_valid_cookie_domain(".example.com", "sub.example.com"));
    }

    #[test]
    fn test_invalid_cookie_domain_public_suffix() {
        assert!(!is_valid_cookie_domain("com", "example.com"));
        assert!(!is_valid_cookie_domain(".com", "example.com"));
        assert!(!is_valid_cookie_domain("co.uk", "example.co.uk"));
    }

    #[test]
    fn test_invalid_cookie_domain_mismatch() {
        assert!(!is_valid_cookie_domain("other.com", "example.com"));
    }
}
