//! QUIC Module Tests
//!
//! Covers:
//! - `QuicConfig` defaults and builder
//! - `QuicConnection` API surface check

use chromenet::quic::{QuicConfig, QuicConnectionBuilder};
use std::time::Duration;

#[test]
fn test_quic_config_defaults() {
    let config = QuicConfig::default();
    assert_eq!(config.idle_timeout, Duration::from_secs(60));
    assert_eq!(config.max_udp_payload_size, 1200);
    assert_eq!(config.initial_max_data, 10 * 1024 * 1024);
    assert!(config.enable_0rtt);
    assert_eq!(config.alpn_protocols, vec!["h3"]);
}

#[test]
fn test_quic_config_builder() {
    let config = QuicConfig::new()
        .idle_timeout(Duration::from_secs(120))
        .initial_rtt(Duration::from_millis(50))
        .max_udp_payload_size(1350)
        .initial_max_data(20 * 1024 * 1024)
        .initial_max_stream_data(2 * 1024 * 1024)
        .enable_0rtt(false)
        .alpn_protocols(vec!["h3-29".to_string()]);

    assert_eq!(config.idle_timeout, Duration::from_secs(120));
    assert_eq!(config.initial_rtt, Duration::from_millis(50));
    assert_eq!(config.max_udp_payload_size, 1350);
    assert_eq!(config.initial_max_data, 20 * 1024 * 1024);
    assert_eq!(config.initial_max_stream_data, 2 * 1024 * 1024);
    assert!(!config.enable_0rtt);
    assert_eq!(config.alpn_protocols, vec!["h3-29"]);
}

#[test]
fn test_quic_connection_builder_api() {
    // Only check API compilation, as connecting requires network
    let _builder = QuicConnectionBuilder::new();
}
