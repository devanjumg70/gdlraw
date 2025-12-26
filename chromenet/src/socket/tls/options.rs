use super::{AlpnProtocol, TlsVersion};
use crate::base::neterror::NetError;
use boring::ssl::{
    CertificateCompressionAlgorithm, ExtensionType, SslConnectorBuilder, SslVerifyMode,
};

/// Builder for `TlsOptions`.
#[must_use]
#[derive(Debug, Clone)]
pub struct TlsOptionsBuilder {
    config: TlsOptions,
}

/// TLS connection configuration options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TlsOptions {
    /// ALPN protocols.
    pub alpn_protocols: Option<Vec<String>>,

    /// Minimum TLS version.
    pub min_tls_version: Option<TlsVersion>,

    /// Maximum TLS version.
    pub max_tls_version: Option<TlsVersion>,

    /// Enable session tickets.
    pub session_ticket: bool,

    /// Cipher suite configuration string.
    pub cipher_list: Option<String>,

    /// Supported curves list.
    pub curves_list: Option<String>,

    /// Supported signature algorithms.
    pub sigalgs_list: Option<String>,

    /// Enable OCSP stapling.
    pub enable_ocsp_stapling: bool,

    /// Enable Signed Certificate Timestamps (SCT).
    pub enable_signed_cert_timestamps: bool,

    /// Enable GREASE.
    pub grease_enabled: Option<bool>,

    /// Permute extensions (requires support in boring).
    pub permute_extensions: Option<bool>,

    /// Enable ECH GREASE (requires support in boring).
    pub enable_ech_grease: bool,

    /// Pre-shared key (PSK).
    pub pre_shared_key: bool,

    /// ALPS protocols (requires support in boring).
    pub alps_protocols: Option<Vec<String>>,

    /// Use new codepoint for ALPS.
    pub alps_use_new_codepoint: bool,

    /// Certificate compression algorithms.
    pub certificate_compression_algorithms: Option<Vec<CertificateCompressionAlgorithm>>,
}

impl Default for TlsOptions {
    fn default() -> Self {
        Self {
            alpn_protocols: Some(vec!["h2".to_string(), "http/1.1".to_string()]),
            min_tls_version: Some(TlsVersion::TLS_1_2),
            max_tls_version: Some(TlsVersion::TLS_1_3),
            session_ticket: true,
            cipher_list: None,
            curves_list: None,
            sigalgs_list: None,
            enable_ocsp_stapling: false,
            enable_signed_cert_timestamps: false,
            grease_enabled: None,
            permute_extensions: None,
            enable_ech_grease: false,
            pre_shared_key: false,
            alps_protocols: None,
            alps_use_new_codepoint: false,
            certificate_compression_algorithms: None,
        }
    }
}

impl TlsOptionsBuilder {
    pub fn new() -> Self {
        Self {
            config: TlsOptions::default(),
        }
    }

    pub fn alpn_protocols(mut self, alpn: &[&str]) -> Self {
        self.config.alpn_protocols = Some(alpn.iter().map(|s| s.to_string()).collect());
        self
    }

    pub fn min_tls_version(mut self, version: TlsVersion) -> Self {
        self.config.min_tls_version = Some(version);
        self
    }

    pub fn max_tls_version(mut self, version: TlsVersion) -> Self {
        self.config.max_tls_version = Some(version);
        self
    }

    pub fn cipher_list(mut self, ciphers: &str) -> Self {
        self.config.cipher_list = Some(ciphers.to_string());
        self
    }

    pub fn curves_list(mut self, curves: &str) -> Self {
        self.config.curves_list = Some(curves.to_string());
        self
    }

    pub fn sigalgs_list(mut self, sigalgs: &str) -> Self {
        self.config.sigalgs_list = Some(sigalgs.to_string());
        self
    }

    pub fn grease_enabled(mut self, enabled: bool) -> Self {
        self.config.grease_enabled = Some(enabled);
        self
    }

    pub fn enable_ocsp_stapling(mut self, enabled: bool) -> Self {
        self.config.enable_ocsp_stapling = enabled;
        self
    }

    pub fn enable_signed_cert_timestamps(mut self, enabled: bool) -> Self {
        self.config.enable_signed_cert_timestamps = enabled;
        self
    }

    pub fn permute_extensions(mut self, permute: bool) -> Self {
        self.config.permute_extensions = Some(permute);
        self
    }

    pub fn enable_ech_grease(mut self, enabled: bool) -> Self {
        self.config.enable_ech_grease = enabled;
        self
    }

    pub fn pre_shared_key(mut self, enabled: bool) -> Self {
        self.config.pre_shared_key = enabled;
        self
    }

    pub fn alps_protocols(mut self, alps: &[&str]) -> Self {
        self.config.alps_protocols = Some(alps.iter().map(|s| s.to_string()).collect());
        self
    }

    pub fn alps_use_new_codepoint(mut self, enabled: bool) -> Self {
        self.config.alps_use_new_codepoint = enabled;
        self
    }

    pub fn certificate_compression_algorithms(
        mut self,
        algs: &[CertificateCompressionAlgorithm],
    ) -> Self {
        self.config.certificate_compression_algorithms = Some(algs.to_vec());
        self
    }

    pub fn build(self) -> TlsOptions {
        self.config
    }
}

impl TlsOptions {
    pub fn builder() -> TlsOptionsBuilder {
        TlsOptionsBuilder::new()
    }

    /// Apply this configuration to an SSL connector builder.
    pub fn apply_to_builder(&self, builder: &mut SslConnectorBuilder) -> Result<(), NetError> {
        // Set TLS versions
        if let Some(min) = self.min_tls_version {
            builder
                .set_min_proto_version(Some(min.0))
                .map_err(|_| NetError::SslProtocolError)?;
        }
        if let Some(max) = self.max_tls_version {
            builder
                .set_max_proto_version(Some(max.0))
                .map_err(|_| NetError::SslProtocolError)?;
        }

        // Set cipher list
        if let Some(ciphers) = &self.cipher_list {
            builder
                .set_cipher_list(ciphers)
                .map_err(|_| NetError::SslProtocolError)?;
        }

        // Set ALPN protocols
        if let Some(protos) = &self.alpn_protocols {
            if !protos.is_empty() {
                let mut alpn_wire = Vec::new();
                for proto in protos {
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
        }

        // Set signature algorithms
        if let Some(sigalgs) = &self.sigalgs_list {
            builder
                .set_sigalgs_list(sigalgs)
                .map_err(|_| NetError::SslProtocolError)?;
        }

        // Set curves/groups
        if let Some(curves) = &self.curves_list {
            builder
                .set_curves_list(curves)
                .map_err(|_| NetError::SslProtocolError)?;
        }

        // Enable session tickets? (default true in boring usually)
        if !self.session_ticket {
            builder.set_num_tickets(0);
        }

        // Certificate verification (use system verifier)
        builder.set_verify(SslVerifyMode::PEER);

        // Note: BoringSSL's boring crate may not expose all features required for
        // full fingerprinting (permute extensions, grease, etc.) via safe API yet.
        // We configure what we can.

        Ok(())
    }
}
