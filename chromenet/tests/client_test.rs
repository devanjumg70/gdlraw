//! Tests for Client API.

use chromenet::client::Client;
use chromenet::emulation::profiles::chrome::Chrome;
use std::time::Duration; // Keep this as it's used later

// === Client Tests ===

#[test]
fn test_client_creation() {
    let _client = Client::new();
}

#[test]
fn test_client_builder() {
    let _client = Client::builder().build();
}

#[test]
fn test_client_with_emulation() {
    let _client = Client::builder().emulation(Chrome::V140).build();
}

#[test]
fn test_client_with_timeout() {
    let _client = Client::builder().timeout(Duration::from_secs(30)).build();
}

#[test]
fn test_client_request_methods() {
    let client = Client::new();

    // All methods should return a RequestBuilder
    let _get = client.get("https://example.com");
    let _post = client.post("https://example.com");
    let _put = client.put("https://example.com");
    let _delete = client.delete("https://example.com");
    let _head = client.head("https://example.com");
    let _patch = client.patch("https://example.com");
}

#[test]
fn test_request_builder_headers() {
    let client = Client::new();

    let _req = client
        .get("https://example.com")
        .header("X-Custom", "value")
        .header(http::header::ACCEPT, "application/json");
}

#[test]
fn test_request_builder_body() {
    let client = Client::new();

    let _req = client
        .post("https://example.com")
        .body(b"test body".to_vec());
}

#[test]
fn test_request_builder_emulation_override() {
    let client = Client::builder().emulation(Chrome::V120).build();

    // Override per-request
    let _req = client.get("https://example.com").emulation(Chrome::V140);
}

#[test]
fn test_client_clone() {
    let client = Client::builder().emulation(Chrome::V140).build();

    let _cloned = client.clone();
}
