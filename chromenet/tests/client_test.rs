//! Tests for Client API.

use chromenet::client::{Client, ClientBuilder};
use chromenet::emulation::profiles::chrome::Chrome;
use chromenet::emulation::{Emulation, EmulationFactory};

// === Client Tests ===

#[test]
fn test_client_new() {
    let client = Client::new();
    // Should create without panic
    assert!(true);
}

#[test]
fn test_client_builder_basic() {
    let client = Client::builder().build();
    assert!(true);
}

#[test]
fn test_client_builder_with_emulation() {
    let client = Client::builder().emulation(Chrome::V140).build();
    assert!(true);
}

#[test]
fn test_client_builder_with_timeout() {
    use std::time::Duration;

    let client = Client::builder().timeout(Duration::from_secs(30)).build();
    assert!(true);
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
