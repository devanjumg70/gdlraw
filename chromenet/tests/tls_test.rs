use boring::ssl::{SslConnector, SslMethod};
use chromenet::socket::tls::TlsConfig;

#[test]
fn test_default_chrome_config() {
    let config = TlsConfig::default_chrome();

    // Check defaults
    assert_eq!(config.cipher_list, "ALL:!aPSK:!ECDSA+SHA1:!3DES");
    assert_eq!(config.alpn_protos, vec!["h2".to_string(), "http/1.1".to_string()]);

    // Check application to builder
    let mut builder = SslConnector::builder(SslMethod::tls()).unwrap();
    let result = config.apply_to_builder(&mut builder);
    assert!(result.is_ok(), "Failed to apply default chrome config to SslConnector");
}

#[test]
fn test_config_application() {
    let mut config = TlsConfig::default_chrome();
    config.alpn_protos = vec!["http/1.1".to_string()]; // Disable h2

    let mut builder = SslConnector::builder(SslMethod::tls()).unwrap();
    let result = config.apply_to_builder(&mut builder);
    assert!(result.is_ok());
}
