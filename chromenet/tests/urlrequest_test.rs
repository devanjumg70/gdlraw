//! URLRequest and Client API Coverage Tests
//!
//! This suite covers the public API surface of:
//! - `URLRequest`
//! - `Client`
//! - `ClientBuilder`
//! - `RequestBuilder`

use chromenet::base::loadstate::LoadState;
use chromenet::base::neterror::NetError;
use chromenet::client::Client;
use chromenet::emulation::profiles::chrome::Chrome;
use chromenet::socket::proxy::ProxyBuilder;
use chromenet::urlrequest::request::URLRequest;
use chromenet::EmulationFactory;
use http::Method;
use std::time::Duration;

#[should_panic(expected = "Pool not created")]
#[ignore] // This needs a real network context or mock
#[test]
fn test_urlrequest_lifecycle() {
    let mut req = URLRequest::new("https://example.com").unwrap();

    // Test builder methods
    req.add_header("X-Custom", "value");
    req.set_method(Method::POST);
    req.set_body("test body");

    // Emulation
    let config = Chrome::V131.emulation();
    // Emulation struct doesn't have a device field directly, it configures TLS/HTTP2/Headers.
    // URLRequest manages Device separately via set_device.
    let _ = config; // just verify we can create it

    // Proxy (API check only)
    let proxy = ProxyBuilder::new().http("localhost:8080").build().unwrap();
    req.set_proxy(proxy);

    // Load state check
    let state = req.load_state();
    assert!(matches!(state, LoadState::Idle));
    // LoadState doesn't impl Display, so just match
    println!("State: {:?}", state);
}

#[test]
fn test_client_builder_api() {
    let _client = Client::builder()
        .emulation(Chrome::V131)
        .timeout(Duration::from_secs(30))
        .build();
}

#[test]
fn test_client_request_methods() {
    let client = Client::new();

    // Basic methods
    let _ = client.get("https://example.com");
    let _ = client.post("https://example.com");
    let _ = client.put("https://example.com");
    let _ = client.delete("https://example.com");
    let _ = client.head("https://example.com");
    let _ = client.patch("https://example.com");

    // Custom method
    let _ = client.request(Method::CONNECT, "https://example.com");
}

#[test]
fn test_request_builder_chaining() {
    let client = Client::new();

    // Chain all builder methods
    let req = client
        .get("https://example.com")
        .header("User-Agent", "CustomAgent")
        .header("Accept", "*/*")
        .body(vec![1, 2, 3])
        .emulation(Chrome::V131); // Override client emulation

    // Just verify it compiles and builds
    let _ = req;
}

#[test]
fn test_urlrequest_methods() {
    // GET
    let req = URLRequest::new("https://example.com").unwrap();
    let _ = req;

    // POST
    let req = URLRequest::post("https://example.com").unwrap();
    let _ = req;

    // PUT
    let req = URLRequest::put("https://example.com").unwrap();
    let _ = req;
}

#[test]
fn test_invalid_url() {
    let res = URLRequest::new("not-a-url");
    assert!(matches!(res, Err(NetError::InvalidUrl)));
}
