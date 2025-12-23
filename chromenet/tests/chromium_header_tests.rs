use chromenet::http::orderedheaders::OrderedHeaderMap;

// Helper to match EXPECT_EQ(string, headers.ToString())
// Chromium's ToString() format: "Key: Value\r\nKey2: Value2\r\n\r\n"
fn assert_headers_eq(headers: &OrderedHeaderMap, expected: &str) {
    let mut output = String::new();
    // We iterate over the map. Keys will be lowercase due to http::HeaderName.
    // Values should preserve case.
    for (k, v) in headers.clone().to_header_map().iter() {
        output.push_str(k.as_str());
        output.push_str(": ");
        output.push_str(v.to_str().unwrap());
        output.push_str("\r\n");
    }
    output.push_str("\r\n");

    // We expect the input `expected` to match strictly (we will provide lowercase keys in test calls).
    assert_eq!(output, expected);
}

#[test]
fn test_has_header() {
    let mut headers = OrderedHeaderMap::new();
    headers.insert("Foo", "bar").unwrap();

    // get() via HeaderName is case-insensitive
    assert!(headers.get("foo").is_some());
    assert!(headers.get("Foo").is_some());
    assert!(headers.get("Fo").is_none());
}

#[test]
fn test_set_header() {
    let mut headers = OrderedHeaderMap::new();
    headers.insert("Foo", "bar").unwrap();
    // http::HeaderName lowercases keys
    assert_headers_eq(&headers, "foo: bar\r\n\r\n");
}

#[test]
fn test_set_multiple_headers() {
    let mut headers = OrderedHeaderMap::new();
    headers.insert("Cookie-Monster", "Nom nom nom").unwrap();
    headers.insert("Domo-Kun", "Loves Chrome").unwrap();
    assert_headers_eq(&headers, "cookie-monster: Nom nom nom\r\ndomo-kun: Loves Chrome\r\n\r\n");
}

#[test]
fn test_set_header_twice() {
    let mut headers = OrderedHeaderMap::new();
    headers.insert("Foo", "bar").unwrap();
    headers.insert("Foo", "bar").unwrap();
    assert_headers_eq(&headers, "foo: bar\r\n\r\n");
}

#[test]
fn test_set_header_twice_case_insensitive() {
    let mut headers = OrderedHeaderMap::new();
    headers.insert("Foo", "bar").unwrap();
    headers.insert("FoO", "Bar").unwrap();
    // Keys lowercase, Value updated to "Bar" (preserved)
    assert_headers_eq(&headers, "foo: Bar\r\n\r\n");
}

#[test]
fn test_remove_header() {
    let mut headers = OrderedHeaderMap::new();
    headers.insert("Foo", "bar").unwrap();
    headers.remove("Foo");
    assert_headers_eq(&headers, "\r\n");
}

#[test]
fn test_remove_header_missing() {
    let mut headers = OrderedHeaderMap::new();
    headers.insert("Foo", "bar").unwrap();
    headers.remove("Bar");
    assert_headers_eq(&headers, "foo: bar\r\n\r\n");
}

#[test]
fn test_remove_header_case_insensitive() {
    let mut headers = OrderedHeaderMap::new();
    headers.insert("Foo", "bar").unwrap();
    headers.insert("All-Your-Base", "Belongs To Chrome").unwrap();
    headers.remove("foo");
    assert_headers_eq(&headers, "all-your-base: Belongs To Chrome\r\n\r\n");
}
