use chromenet::http::orderedheaders::OrderedHeaderMap;

#[test]
fn test_ordered_headers_insertion_order() {
    let mut map = OrderedHeaderMap::new();

    // Insert in specific order
    map.insert("Host", "example.com").unwrap();
    map.insert("Connection", "keep-alive").unwrap();
    map.insert("User-Agent", "Chromenet/0.1").unwrap();
    map.insert("Accept", "*/*").unwrap();

    // Verify internal order
    let header_map = map.to_header_map();
    let mut iter = header_map.iter();

    // Note: http::HeaderMap iteration order is generally insertion order for distinct keys.
    // But it's technically an implementation detail of the `http` crate (Robin Hood Hashing with linear probe?)
    // Actually, `http` 1.0 logic states: "The order is preserved."

    assert_eq!(iter.next().unwrap().0, "host");
    assert_eq!(iter.next().unwrap().0, "connection");
    assert_eq!(iter.next().unwrap().0, "user-agent");
    assert_eq!(iter.next().unwrap().0, "accept");
}

#[test]
fn test_ordered_headers_update_preserves_order() {
    let mut map = OrderedHeaderMap::new();

    map.insert("A", "1").unwrap();
    map.insert("B", "2").unwrap();
    map.insert("C", "3").unwrap();

    // Update B
    map.insert("B", "22").unwrap();

    let header_map = map.to_header_map();
    let mut iter = header_map.iter();

    assert_eq!(iter.next().unwrap().0, "a");

    let b = iter.next().unwrap();
    assert_eq!(b.0, "b"); // Should still be second
    assert_eq!(b.1, "22");

    let c = iter.next().unwrap();
    assert_eq!(c.0, "c");
}
