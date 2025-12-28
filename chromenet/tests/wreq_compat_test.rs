//! Wreq Compatibility Tests
//!
//! These tests are derived from wreq/tests and wreq-util/tests to verify
//! chromenet's compatibility with wreq-like patterns.
//!
//! Status: Documents what works and what doesn't vs wreq patterns

use chromenet::base::neterror::NetError;
use chromenet::cookies::monster::CookieMonster;
use chromenet::emulation::EmulationFactory;
use url::Url;

// ============================================================================
// COOKIE TESTS (derived from wreq/tests/cookie.rs)
// ============================================================================

/// Test: Cookie parsing and storage (wreq: cookie_store_simple)
#[test]
fn test_cookie_store_simple() {
    let jar = CookieMonster::new();
    let url = Url::parse("http://example.com/").unwrap();

    jar.parse_and_save_cookie(&url, "key=val; HttpOnly");

    let cookies = jar.get_cookies_for_url(&url);
    assert_eq!(cookies.len(), 1);
    assert_eq!(cookies[0].name, "key");
    assert_eq!(cookies[0].value, "val");
    assert!(cookies[0].http_only);
}

/// Test: Cookie overwrite (wreq: cookie_store_overwrite_existing)
#[test]
fn test_cookie_store_overwrite() {
    let jar = CookieMonster::new();
    let url = Url::parse("http://example.com/").unwrap();

    jar.parse_and_save_cookie(&url, "key=val1");
    jar.parse_and_save_cookie(&url, "key=val2");

    let cookies = jar.get_cookies_for_url(&url);
    assert_eq!(cookies.len(), 1);
    assert_eq!(cookies[0].value, "val2");
}

/// Test: Cookie path matching (wreq: cookie_store_path)
#[test]
fn test_cookie_store_path() {
    let jar = CookieMonster::new();
    let root_url = Url::parse("http://example.com/").unwrap();
    let subpath_url = Url::parse("http://example.com/subpath").unwrap();

    jar.parse_and_save_cookie(&root_url, "key=val; Path=/subpath");

    let root_cookies = jar.get_cookies_for_url(&root_url);
    assert_eq!(root_cookies.len(), 0);

    let subpath_cookies = jar.get_cookies_for_url(&subpath_url);
    assert_eq!(subpath_cookies.len(), 1);
}

/// Test: Cookie domain matching
#[test]
fn test_cookie_domain_matching() {
    let jar = CookieMonster::new();
    let url = Url::parse("http://sub.example.com/").unwrap();

    jar.parse_and_save_cookie(&url, "key=val; Domain=example.com");

    let cookies = jar.get_cookies_for_url(&url);
    assert_eq!(cookies.len(), 1);

    let parent_url = Url::parse("http://example.com/").unwrap();
    let parent_cookies = jar.get_cookies_for_url(&parent_url);
    assert_eq!(parent_cookies.len(), 1);
}

/// Test: Cookie secure attribute
#[test]
fn test_cookie_secure_attribute() {
    let jar = CookieMonster::new();
    let https_url = Url::parse("https://example.com/").unwrap();
    let http_url = Url::parse("http://example.com/").unwrap();

    jar.parse_and_save_cookie(&https_url, "key=val; Secure");

    let https_cookies = jar.get_cookies_for_url(&https_url);
    assert_eq!(https_cookies.len(), 1);

    let http_cookies = jar.get_cookies_for_url(&http_url);
    assert_eq!(http_cookies.len(), 0);
}

/// Test: Cookie expiration
#[test]
fn test_cookie_expiration() {
    let jar = CookieMonster::new();
    let url = Url::parse("http://example.com/").unwrap();

    jar.parse_and_save_cookie(&url, "key=val; Expires=Wed, 21 Oct 2015 07:28:00 GMT");

    let cookies = jar.get_cookies_for_url(&url);
    assert_eq!(cookies.len(), 0);
}

/// Test: Multiple cookies
#[test]
fn test_multiple_cookies() {
    let jar = CookieMonster::new();
    let url = Url::parse("http://example.com/").unwrap();

    jar.parse_and_save_cookie(&url, "cookie1=value1");
    jar.parse_and_save_cookie(&url, "cookie2=value2");
    jar.parse_and_save_cookie(&url, "cookie3=value3");

    let cookies = jar.get_cookies_for_url(&url);
    assert_eq!(cookies.len(), 3);
}

/// Test: Netscape format export/import
#[test]
fn test_netscape_export_import() {
    let jar1 = CookieMonster::new();
    let url = Url::parse("http://example.com/").unwrap();

    jar1.parse_and_save_cookie(&url, "key1=val1");
    jar1.parse_and_save_cookie(&url, "key2=val2");

    let exported = jar1.export_netscape(None);

    let jar2 = CookieMonster::new();
    let count = jar2.import_netscape(&exported);

    assert_eq!(count, 2);
    assert_eq!(jar2.total_cookie_count(), 2);
}

// ============================================================================
// EMULATION TESTS (derived from wreq/tests/emulation.rs)
// ============================================================================

/// Test: Chrome profile via factory
#[test]
fn test_chrome_profile_factory() {
    use chromenet::emulation::profiles::chrome::Chrome;

    let emulation = Chrome::V131.emulation();
    assert!(emulation.tls_options.is_some());
    assert!(emulation.http2_options.is_some());
}

/// Test: Firefox profile via factory
#[test]
fn test_firefox_profile_factory() {
    use chromenet::emulation::profiles::firefox::Firefox;

    let emulation = Firefox::V133.emulation();
    assert!(emulation.tls_options.is_some());
}

/// Test: Safari profile via factory
#[test]
fn test_safari_profile_factory() {
    use chromenet::emulation::profiles::safari::Safari;

    let emulation = Safari::V18.emulation();
    assert!(emulation.tls_options.is_some());
}

/// Test: Edge profile via factory
#[test]
fn test_edge_profile_factory() {
    use chromenet::emulation::profiles::edge::Edge;

    let emulation = Edge::V127.emulation();
    assert!(emulation.tls_options.is_some());
}

/// Test: OkHttp profile
#[test]
fn test_okhttp_profile_factory() {
    use chromenet::emulation::profiles::okhttp::OkHttp;

    let emulation = OkHttp::V5.emulation();
    assert!(emulation.tls_options.is_some());
}

// ============================================================================
// ERROR HANDLING TESTS
// ============================================================================

/// Test: NetError variants exist
#[test]
fn test_neterror_variants() {
    let _ = NetError::ConnectionFailed;
    let _ = NetError::ConnectionReset;
    let _ = NetError::ConnectionTimedOut;
    let _ = NetError::InvalidUrl;
    let _ = NetError::TooManyRedirects;
}

/// Test: NetError From implementations
#[test]
fn test_neterror_from_impls() {
    use std::io;

    let io_err = io::Error::new(io::ErrorKind::ConnectionRefused, "refused");
    let net_err: NetError = io_err.into();
    assert!(matches!(net_err, NetError::ConnectionRefused));

    let url_err = Url::parse("not-a-url").unwrap_err();
    let net_err: NetError = url_err.into();
    assert!(matches!(net_err, NetError::InvalidUrl));
}

// ============================================================================
// DNS TESTS
// ============================================================================

/// Test: DNS resolver types exist
#[test]
fn test_dns_resolver_types() {
    use chromenet::dns::{GaiResolver, HickoryResolver, Name};

    let _name = Name::new("example.com");
    let _gai = GaiResolver::new();
    let _hickory = HickoryResolver::new();
}

// ============================================================================
// TLS TESTS
// ============================================================================

/// Test: TLS config exists with cipher list
#[test]
fn test_tls_config_exists() {
    use chromenet::socket::tls::TlsConfig;

    let config = TlsConfig::default_chrome();
    assert!(!config.cipher_list.is_empty());
}

/// Test: HSTS store operations
#[test]
fn test_hsts_store_operations() {
    use chromenet::tls::HstsStore;

    let store = HstsStore::new();
    store.add_preloaded("example.com", true);
    assert!(store.should_upgrade("sub.example.com"));
}

// ============================================================================
// REDIRECT/ERROR TESTS
// ============================================================================

/// Test: Redirect error variants exist
#[test]
fn test_redirect_error_variants() {
    let _ = NetError::TooManyRedirects;
    let _ = NetError::RedirectCycleDetected;
}

// ============================================================================
// CONNECTION POOL TESTS
// ============================================================================

/// Test: Pool can be created
#[test]
fn test_pool_creation() {
    use chromenet::socket::pool::ClientSocketPool;

    let pool = ClientSocketPool::new(None);
    let _ = pool;
}

// ============================================================================
// WEBSOCKET TESTS
// ============================================================================

/// Test: WebSocket types exist
#[test]
fn test_websocket_types() {
    use chromenet::ws::{CloseCode, CloseFrame, Message};

    let _ = Message::Text("hello".into());
    let _ = Message::Binary(bytes::Bytes::from_static(b"data"));
    let _ = CloseCode::NORMAL;
    let _ = CloseFrame {
        code: CloseCode::NORMAL,
        reason: "bye".into(),
    };
}

// ============================================================================
// MULTIPART TESTS
// ============================================================================

/// Test: Multipart form types exist
#[test]
fn test_multipart_types() {
    use chromenet::http::multipart::{Form, Part};

    let mut form = Form::new();
    form = form.text("field", "value");
    form = form.part(
        "file",
        Part::bytes(b"content".to_vec()).file_name("test.txt"),
    );

    // Form should exist
    let _ = form;
}

// ============================================================================
// HTTP CACHE TESTS
// ============================================================================

/// Test: HTTP cache types exist
#[test]
fn test_http_cache_types() {
    use chromenet::http::HttpCache;

    let cache = HttpCache::new();
    let _ = cache;
}
