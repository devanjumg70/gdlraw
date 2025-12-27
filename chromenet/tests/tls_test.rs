//! Tests for TLS options and emulation.

use boring::ssl::{SslConnector, SslMethod};
use chromenet::socket::tls::{AlpnProtocol, TlsConfig, TlsOptions, TlsVersion};

// === TlsOptions Tests ===

#[test]
fn test_tls_options_default() {
    let opts = TlsOptions::default();

    // Default ALPN: H2, HTTP1
    assert!(opts.alpn_protocols.is_some());
    let alpn = opts.alpn_protocols.as_ref().unwrap();
    assert_eq!(alpn.len(), 2);

    // Default versions: TLS 1.2 - 1.3
    assert_eq!(opts.min_tls_version, Some(TlsVersion::TLS_1_2));
    assert_eq!(opts.max_tls_version, Some(TlsVersion::TLS_1_3));

    // Default session ticket enabled
    assert!(opts.session_ticket);
    assert!(opts.psk_dhe_ke);
    assert!(opts.renegotiation);
}

#[test]
fn test_tls_options_builder_chain() {
    let opts = TlsOptions::builder()
        .alpn_protocols([AlpnProtocol::HTTP2])
        .min_tls_version(TlsVersion::TLS_1_3)
        .cipher_list("TLS_AES_128_GCM_SHA256")
        .curves_list("X25519")
        .grease_enabled(true)
        .permute_extensions(true)
        .session_ticket(false)
        .build();

    // Verify values
    assert!(opts.alpn_protocols.is_some());
    assert_eq!(opts.min_tls_version, Some(TlsVersion::TLS_1_3));
    assert_eq!(opts.cipher_list.as_deref(), Some("TLS_AES_128_GCM_SHA256"));
    assert_eq!(opts.curves_list.as_deref(), Some("X25519"));
    assert_eq!(opts.grease_enabled, Some(true));
    assert_eq!(opts.permute_extensions, Some(true));
    assert!(!opts.session_ticket);
}

#[test]
fn test_tls_options_apply_to_builder() {
    let opts = TlsOptions::builder()
        .alpn_protocols([AlpnProtocol::HTTP2, AlpnProtocol::HTTP1])
        .min_tls_version(TlsVersion::TLS_1_2)
        .grease_enabled(true)
        .build();

    let mut builder = SslConnector::builder(SslMethod::tls()).unwrap();
    let result = opts.apply_to_builder(&mut builder);
    assert!(result.is_ok(), "Failed to apply TlsOptions: {:?}", result);
}

#[test]
fn test_tls_options_builder_preserves_defaults() {
    // Building without any changes should preserve defaults
    let default = TlsOptions::default();
    let built = TlsOptions::builder().build();

    assert_eq!(default.session_ticket, built.session_ticket);
    assert_eq!(default.psk_dhe_ke, built.psk_dhe_ke);
    assert_eq!(default.renegotiation, built.renegotiation);
}

#[test]
fn test_alpn_wire_format() {
    let wire = AlpnProtocol::encode_wire_format(&[AlpnProtocol::HTTP2, AlpnProtocol::HTTP1]);
    // h2 = 2 bytes, http/1.1 = 8 bytes
    // Format: len(h2), "h2", len(http/1.1), "http/1.1"
    assert_eq!(wire, b"\x02h2\x08http/1.1");
}

#[test]
fn test_tls_version_hash() {
    use std::collections::HashSet;

    let mut versions = HashSet::new();
    versions.insert(TlsVersion::TLS_1_0);
    versions.insert(TlsVersion::TLS_1_1);
    versions.insert(TlsVersion::TLS_1_2);
    versions.insert(TlsVersion::TLS_1_3);

    assert_eq!(versions.len(), 4);
    assert!(versions.contains(&TlsVersion::TLS_1_2));
}

// === TlsConfig Legacy Tests (kept for compatibility) ===

#[test]
fn test_default_chrome_config() {
    let config = TlsConfig::default_chrome();

    assert_eq!(
        config.alpn_protos,
        vec!["h2".to_string(), "http/1.1".to_string()]
    );

    let mut builder = SslConnector::builder(SslMethod::tls()).unwrap();
    let result = config.apply_to_builder(&mut builder);
    assert!(result.is_ok());
}

#[test]
fn test_config_application() {
    let mut config = TlsConfig::default_chrome();
    config.alpn_protos = vec!["http/1.1".to_string()];

    let mut builder = SslConnector::builder(SslMethod::tls()).unwrap();
    let result = config.apply_to_builder(&mut builder);
    assert!(result.is_ok());
}
