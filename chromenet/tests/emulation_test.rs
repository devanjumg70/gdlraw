//! Tests for emulation module.

use chromenet::emulation::profiles::chrome::Chrome;
use chromenet::emulation::{Emulation, EmulationFactory, Http1Options, Http2Options};
use chromenet::socket::tls::TlsOptions;

// === Emulation Tests ===

#[test]
fn test_emulation_default() {
    let emu = Emulation::default();
    assert!(emu.tls_options().is_none());
    assert!(emu.http1_options().is_none());
    assert!(emu.http2_options().is_none());
    assert!(emu.headers().is_empty());
}

#[test]
fn test_emulation_builder_tls() {
    let tls = TlsOptions::builder().grease_enabled(true).build();

    let emu = Emulation::builder().tls_options(tls).build();

    assert!(emu.tls_options().is_some());
    assert_eq!(emu.tls_options().unwrap().grease_enabled, Some(true));
}

#[test]
fn test_emulation_builder_h2() {
    let h2 = Http2Options::builder()
        .initial_window_size(6291456)
        .max_header_list_size(262144)
        .build();

    let emu = Emulation::builder().http2_options(h2).build();

    assert!(emu.http2_options().is_some());
    let opts = emu.http2_options().unwrap();
    assert_eq!(opts.initial_window_size, Some(6291456));
    assert_eq!(opts.max_header_list_size, Some(262144));
}

#[test]
fn test_emulation_builder_headers() {
    let emu = Emulation::builder()
        .header(http::header::USER_AGENT, "Test/1.0")
        .header(http::header::ACCEPT, "text/html")
        .build();

    assert_eq!(emu.headers().len(), 2);
    assert!(emu.headers().contains_key(http::header::USER_AGENT));
    assert!(emu.headers().contains_key(http::header::ACCEPT));
}

#[test]
fn test_emulation_into_parts() {
    let tls = TlsOptions::default();
    let h1 = Http1Options::default();

    let emu = Emulation::builder()
        .tls_options(tls)
        .http1_options(h1)
        .build();

    let (tls_opt, h1_opt, h2_opt, headers) = emu.into_parts();
    assert!(tls_opt.is_some());
    assert!(h1_opt.is_some());
    assert!(h2_opt.is_none());
    assert!(headers.is_empty());
}

// === EmulationFactory Tests ===

#[test]
fn test_emulation_factory_from_tls() {
    let tls = TlsOptions::builder()
        .cipher_list("TLS_AES_128_GCM_SHA256")
        .build();

    let emu = tls.emulation();
    assert!(emu.tls_options().is_some());
}

#[test]
fn test_emulation_factory_from_h2() {
    let h2 = Http2Options::builder().initial_window_size(1048576).build();

    let emu = h2.emulation();
    assert!(emu.http2_options().is_some());
}

// === Chrome Profile Tests ===

#[test]
fn test_chrome_default_is_latest() {
    assert_eq!(Chrome::default(), Chrome::V140);
}

#[test]
fn test_chrome_v140_emulation() {
    let emu = Chrome::V140.emulation();

    // Should have TLS options
    assert!(emu.tls_options().is_some());
    let tls = emu.tls_options().unwrap();
    assert!(tls.grease_enabled.unwrap_or(false));

    // Should have H2 options
    assert!(emu.http2_options().is_some());

    // Should have headers
    assert!(!emu.headers().is_empty());
    assert!(emu.headers().contains_key(http::header::USER_AGENT));
}

#[test]
fn test_all_chrome_versions_valid() {
    let versions = [
        Chrome::V120,
        Chrome::V124,
        Chrome::V128,
        Chrome::V131,
        Chrome::V135,
        Chrome::V140,
    ];

    for version in versions {
        let emu = version.emulation();
        assert!(
            emu.tls_options().is_some(),
            "{:?} missing TLS options",
            version
        );
        assert!(
            emu.http2_options().is_some(),
            "{:?} missing H2 options",
            version
        );
        assert!(!emu.headers().is_empty(), "{:?} missing headers", version);
    }
}

// === Http1Options Tests ===

#[test]
fn test_h1_options_builder() {
    let opts = Http1Options::builder()
        .title_case_headers(true)
        .preserve_header_order(true)
        .build();

    assert!(opts.title_case_headers);
    assert!(opts.preserve_header_order);
}

// === Http2Options Tests ===

#[test]
fn test_h2_options_builder() {
    let opts = Http2Options::builder()
        .initial_window_size(15728640)
        .max_frame_size(16384)
        .max_concurrent_streams(100)
        .header_table_size(65536)
        .enable_push(false)
        .build();

    assert_eq!(opts.initial_window_size, Some(15728640));
    assert_eq!(opts.max_frame_size, Some(16384));
    assert_eq!(opts.max_concurrent_streams, Some(100));
    assert_eq!(opts.header_table_size, Some(65536));
    assert_eq!(opts.enable_push, Some(false));
}

// === Firefox Profile Tests ===

#[test]
fn test_firefox_default() {
    use chromenet::emulation::profiles::Firefox;
    assert_eq!(Firefox::default(), Firefox::V145);
}

#[test]
fn test_firefox_emulation() {
    use chromenet::emulation::profiles::Firefox;
    let emu = Firefox::V145.emulation();

    // Firefox should have TLS without GREASE
    assert!(emu.tls_options().is_some());
    let tls = emu.tls_options().unwrap();
    assert_eq!(tls.grease_enabled, Some(false));

    // Should have H2 with smaller window size than Chrome
    assert!(emu.http2_options().is_some());
    let h2 = emu.http2_options().unwrap();
    assert_eq!(h2.initial_window_size, Some(131072));

    // Should have headers
    assert!(!emu.headers().is_empty());
}

#[test]
fn test_all_firefox_versions() {
    use chromenet::emulation::profiles::Firefox;
    let versions = [
        Firefox::V128,
        Firefox::V133,
        Firefox::V135,
        Firefox::V140,
        Firefox::V145,
    ];

    for v in versions {
        let emu = v.emulation();
        assert!(emu.tls_options().is_some(), "{:?} missing TLS", v);
        assert!(emu.http2_options().is_some(), "{:?} missing H2", v);
        assert!(!emu.headers().is_empty(), "{:?} missing headers", v);
    }
}

// === Safari Profile Tests ===

#[test]
fn test_safari_default() {
    use chromenet::emulation::profiles::Safari;
    assert_eq!(Safari::default(), Safari::V18_2);
}

#[test]
fn test_safari_emulation() {
    use chromenet::emulation::profiles::Safari;
    let emu = Safari::V18.emulation();

    // Safari should have TLS without GREASE
    assert!(emu.tls_options().is_some());
    let tls = emu.tls_options().unwrap();
    assert_eq!(tls.grease_enabled, Some(false));

    // Should have H2 with 4MB window
    assert!(emu.http2_options().is_some());
    let h2 = emu.http2_options().unwrap();
    assert_eq!(h2.initial_window_size, Some(4194304));

    assert!(!emu.headers().is_empty());
}

#[test]
fn test_all_safari_versions() {
    use chromenet::emulation::profiles::Safari;
    let versions = [
        Safari::V17,
        Safari::V17_5,
        Safari::V18,
        Safari::V18_2,
        Safari::IOS17,
        Safari::IOS18,
    ];

    for v in versions {
        let emu = v.emulation();
        assert!(emu.tls_options().is_some(), "{:?} missing TLS", v);
        assert!(!emu.headers().is_empty(), "{:?} missing headers", v);
    }
}

// === Edge Profile Tests ===

#[test]
fn test_edge_default() {
    use chromenet::emulation::profiles::Edge;
    assert_eq!(Edge::default(), Edge::V140);
}

#[test]
fn test_edge_emulation() {
    use chromenet::emulation::profiles::Edge;
    let emu = Edge::V140.emulation();

    // Edge is Chromium-based, should have GREASE
    assert!(emu.tls_options().is_some());
    let tls = emu.tls_options().unwrap();
    assert!(tls.grease_enabled.unwrap_or(false));

    // Should have headers with Edge branding
    assert!(!emu.headers().is_empty());
    let ua = emu.headers().get(http::header::USER_AGENT).unwrap();
    assert!(ua.to_str().unwrap().contains("Edg/"));
}

#[test]
fn test_all_edge_versions() {
    use chromenet::emulation::profiles::Edge;
    let versions = [Edge::V120, Edge::V131, Edge::V135, Edge::V140];

    for v in versions {
        let emu = v.emulation();
        assert!(emu.tls_options().is_some(), "{:?} missing TLS", v);
        assert!(!emu.headers().is_empty(), "{:?} missing headers", v);
    }
}
