//! Header Stress Tests
//!
//! Covers:
//! - `OrderedHeaderMap` ordering persistence with large counts
//! - `CaseSensitiveHeaders` exact casing retention

use chromenet::http::orderedheaders::{CaseSensitiveHeaders, OrderedHeaderMap};

#[test]
fn test_ordered_headers_stress() {
    let mut map = OrderedHeaderMap::default();
    let count = 1000;

    // Insert 1000 headers: X-0, X-1, ...
    for i in 0..count {
        map.insert(&format!("X-{}", i), "value").unwrap();
    }

    // Convert to standard map
    let standard = map.to_header_map();

    // Check all exist
    assert_eq!(standard.len(), count);

    // standard HeaderMap iterates in *insertion order*?
    // Yes, http::HeaderMap preserves order for same-name keys, but different keys implementation detail.
    // However, `OrderedHeaderMap` explicitly stores a `Vec<HeaderName>` to guarantee it.
    // The test `test_preserves_insertion_order` in `orderedheaders.rs` checked this.
    //
    // Here we just verify it handles volume without crashing or losing data.
    for i in 0..count {
        let key = format!("X-{}", i);
        assert!(standard.contains_key(&key));
    }
}

#[test]
fn test_case_sensitive_headers_exact_match() {
    let mut headers = CaseSensitiveHeaders::default();

    // Insert mixed case
    headers.insert("Content-Type", "json");
    headers.insert("User-AGENT", "test");
    headers.insert("x-custom-HEADER", "val");

    // Verify stored keys are exact
    let stored: Vec<_> = headers.iter().map(|(k, _)| k).collect();
    assert!(stored.contains(&"Content-Type"));
    assert!(stored.contains(&"User-AGENT"));
    assert!(stored.contains(&"x-custom-HEADER"));
    assert!(!stored.contains(&"content-type")); // Should NOT allow lowercase raw access via iter

    // Verify lookup is case-insensitive
    assert_eq!(headers.get("content-type"), Some("json"));
    assert_eq!(headers.get("USER-AGENT"), Some("test"));
}
