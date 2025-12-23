use crate::base::neterror::NetError;
use boring::ssl::{SslConnectorBuilder, SslVerifyMode};

/// Configuration for TLS Client Hello fingerprinting.
#[derive(Debug, Clone)]
pub struct TlsConfig {
    pub min_version: Option<u16>, // e.g. TLS1_2_VERSION
    pub max_version: Option<u16>,
    pub cipher_list: String,
    pub alpn_protos: Vec<String>,
    pub enable_grease: bool,
    pub enable_ocsp_stapling: bool,
    pub enable_sct: bool,
    pub curves: Vec<i32>,     // NID values
    pub sigalgs: Vec<String>, // OpenSSL names "ECDSA+SHA256"
}

impl TlsConfig {
    pub fn default_chrome() -> Self {
        Self {
            min_version: None, // Uses lib default (usually TLS 1.2)
            max_version: None,
            // Chromium's default cipher list: "ALL:!aPSK:!ECDSA+SHA1:!3DES"
            cipher_list: "ALL:!aPSK:!ECDSA+SHA1:!3DES".to_string(),
            alpn_protos: vec!["h2".to_string(), "http/1.1".to_string()],
            enable_grease: true,
            enable_ocsp_stapling: true,
            enable_sct: true,
            // Default curves (X25519, P-256, P-384) - We might rely on BoringSSL defaults if API is hard
            curves: vec![],
            sigalgs: vec![],
        }
    }

    pub fn apply_to_builder(&self, builder: &mut SslConnectorBuilder) -> Result<(), NetError> {
        // Ciphers
        builder.set_cipher_list(&self.cipher_list).map_err(|_| NetError::SslProtocolError)?;

        // ALPN
        let mut alpn_wire = Vec::new();
        for proto in &self.alpn_protos {
            if proto.len() > 255 {
                return Err(NetError::SslProtocolError);
            }
            alpn_wire.push(proto.len() as u8);
            alpn_wire.extend_from_slice(proto.as_bytes());
        }
        if !alpn_wire.is_empty() {
            builder.set_alpn_protos(&alpn_wire).map_err(|_| NetError::SslProtocolError)?;
        }

        // GREASE - Note: BoringSSL enables GREASE by default in SSL_CTX_new?
        // Chromium explicitly calls SSL_CTX_set_grease_enabled(ctx, 1).

        // Verify Mode (Default is PEER, but we need to handle CertVerifier separately if mimicking Chrome's verifier)
        // For now, use system verifier.
        builder.set_verify(SslVerifyMode::PEER);

        Ok(())
    }
}
