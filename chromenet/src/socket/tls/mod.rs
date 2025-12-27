//! TLS module providing fingerprinting and configuration.
//!
//! Combines Chromium-style TLS configuration with wreq emulation capabilities.

use crate::base::neterror::NetError;
use boring::ssl::{SslConnectorBuilder, SslVerifyMode, SslVersion};

pub mod impersonate;
pub mod options;

// Re-export all types from options
pub use self::impersonate::ImpersonateTarget;
pub use self::options::{
    AlpnProtocol, AlpsProtocol, CertCompressAlg, TlsOptions, TlsOptionsBuilder, TlsVersion,
};

/// Configuration for TLS Client Hello fingerprinting.
/// Matches Chromium's TLS configuration for accurate fingerprinting.
#[derive(Debug, Clone)]
pub struct TlsConfig {
    pub min_version: Option<SslVersion>,
    pub max_version: Option<SslVersion>,
    pub cipher_list: String,
    pub alpn_protos: Vec<String>,
    pub enable_grease: bool,
    pub enable_ocsp_stapling: bool,
    pub enable_signed_cert_timestamps: bool,
    pub curves: Vec<String>,
    pub sigalgs: String,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self::default_chrome()
    }
}

impl TlsConfig {
    /// Create a TLS configuration matching Chrome's defaults.
    pub fn default_chrome() -> Self {
        Self {
            min_version: Some(SslVersion::TLS1_2),
            max_version: Some(SslVersion::TLS1_3),
            cipher_list:
                "TLS_AES_128_GCM_SHA256:TLS_AES_256_GCM_SHA384:TLS_CHACHA20_POLY1305_SHA256:\
                ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:\
                ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384:\
                ECDHE-ECDSA-CHACHA20-POLY1305:ECDHE-RSA-CHACHA20-POLY1305:\
                ECDHE-RSA-AES128-SHA:ECDHE-RSA-AES256-SHA:\
                AES128-GCM-SHA256:AES256-GCM-SHA384:AES128-SHA:AES256-SHA"
                    .to_string(),
            alpn_protos: vec!["h2".to_string(), "http/1.1".to_string()],
            enable_grease: true,
            enable_ocsp_stapling: true,
            enable_signed_cert_timestamps: true,
            curves: vec![
                "X25519".to_string(),
                "P-256".to_string(),
                "P-384".to_string(),
            ],
            sigalgs: "ECDSA+SHA256:RSA-PSS+SHA256:RSA+SHA256:\
                ECDSA+SHA384:RSA-PSS+SHA384:RSA+SHA384:\
                RSA-PSS+SHA512:RSA+SHA512"
                .to_string(),
        }
    }

    /// Apply this configuration to an SSL connector builder.
    pub fn apply_to_builder(&self, builder: &mut SslConnectorBuilder) -> Result<(), NetError> {
        if let Some(min) = self.min_version {
            builder
                .set_min_proto_version(Some(min))
                .map_err(|_| NetError::SslProtocolError)?;
        }
        if let Some(max) = self.max_version {
            builder
                .set_max_proto_version(Some(max))
                .map_err(|_| NetError::SslProtocolError)?;
        }

        builder
            .set_cipher_list(&self.cipher_list)
            .map_err(|_| NetError::SslProtocolError)?;

        if !self.alpn_protos.is_empty() {
            let mut alpn_wire = Vec::new();
            for proto in &self.alpn_protos {
                if proto.len() > 255 {
                    return Err(NetError::SslProtocolError);
                }
                alpn_wire.push(proto.len() as u8);
                alpn_wire.extend_from_slice(proto.as_bytes());
            }
            builder
                .set_alpn_protos(&alpn_wire)
                .map_err(|_| NetError::SslProtocolError)?;
        }

        if !self.sigalgs.is_empty() {
            builder
                .set_sigalgs_list(&self.sigalgs)
                .map_err(|_| NetError::SslProtocolError)?;
        }

        if !self.curves.is_empty() {
            let curves_str = self.curves.join(":");
            builder
                .set_curves_list(&curves_str)
                .map_err(|_| NetError::SslProtocolError)?;
        }

        builder.set_verify(SslVerifyMode::PEER);

        Ok(())
    }

    /// Check if SNI should be set for this host.
    /// Per RFC 6066, SNI MUST NOT be set for raw IP addresses.
    pub fn should_set_sni(host: &str) -> bool {
        host.parse::<std::net::IpAddr>().is_err()
    }
}
