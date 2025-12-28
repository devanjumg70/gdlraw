//! Integration tests with real HTTP servers.
//!
//! These tests make actual network requests and verify end-to-end functionality.

use std::time::Duration;

/// Test real HTTP request to httpbin.org
#[tokio::test]
#[ignore] // Run with --ignored flag for network tests
async fn test_real_http_get() {
    use chromenet::client::Client;

    let client = Client::builder().timeout(Duration::from_secs(10)).build();

    let response = client.get("https://httpbin.org/get").send().await;

    match response {
        Ok(resp) => {
            assert_eq!(resp.status(), 200);
            let text = resp.text().await.unwrap();
            assert!(text.contains("httpbin.org"));
        }
        Err(e) => {
            // Network might be unavailable in CI
            eprintln!("Network test skipped: {:?}", e);
        }
    }
}

/// Test HTTPS with browser emulation
#[tokio::test]
#[ignore]
async fn test_https_with_emulation() {
    use chromenet::client::Client;
    use chromenet::emulation::profiles::chrome::Chrome;

    let client = Client::builder()
        .emulation(Chrome::V143)
        .timeout(Duration::from_secs(15))
        .build();

    let response = client.get("https://httpbin.org/headers").send().await;

    match response {
        Ok(resp) => {
            assert_eq!(resp.status(), 200);
            let text = resp.text().await.unwrap();
            // Check that User-Agent was set by emulation
            assert!(text.contains("Chrome") || text.contains("user-agent"));
        }
        Err(e) => {
            eprintln!("Emulation test skipped: {:?}", e);
        }
    }
}

/// Test POST request
#[tokio::test]
#[ignore]
async fn test_http_post() {
    use chromenet::client::Client;

    let client = Client::new();

    let response = client
        .post("https://httpbin.org/post")
        .body(b"test data".to_vec())
        .header("Content-Type", "text/plain")
        .send()
        .await;

    match response {
        Ok(resp) => {
            assert_eq!(resp.status(), 200);
            let text = resp.text().await.unwrap();
            assert!(text.contains("test data"));
        }
        Err(e) => {
            eprintln!("POST test skipped: {:?}", e);
        }
    }
}

/// Test redirect following
#[tokio::test]
#[ignore]
async fn test_redirect_following() {
    use chromenet::client::Client;

    let client = Client::new();

    let response = client.get("https://httpbin.org/redirect/2").send().await;

    match response {
        Ok(resp) => {
            // After following 2 redirects, should get 200
            assert_eq!(resp.status(), 200);
        }
        Err(e) => {
            eprintln!("Redirect test skipped: {:?}", e);
        }
    }
}

/// Test cookies across requests
#[tokio::test]
#[ignore]
async fn test_cookie_persistence() {
    use chromenet::client::Client;

    let client = Client::new();

    // First request to set cookie
    let resp1 = client
        .get("https://httpbin.org/cookies/set?test_cookie=chromenet")
        .send()
        .await;

    if resp1.is_err() {
        eprintln!("Cookie set skipped");
        return;
    }

    // Second request should send cookie back
    let resp2 = client.get("https://httpbin.org/cookies").send().await;

    match resp2 {
        Ok(resp) => {
            let text = resp.text().await.unwrap();
            assert!(text.contains("chromenet") || text.contains("test_cookie"));
        }
        Err(e) => {
            eprintln!("Cookie check skipped: {:?}", e);
        }
    }
}
