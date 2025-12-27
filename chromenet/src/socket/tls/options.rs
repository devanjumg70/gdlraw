//! TLS connection configuration options.
//!
//! Provides fine-grained control over TLS behavior for browser fingerprint emulation.
//! Based on wreq's comprehensive TlsOptions with chromenet architecture integration.

use crate::base::neterror::NetError;
use boring::ssl::{
    CertificateCompressionAlgorithm, ExtensionType, SslConnectorBuilder, SslVerifyMode,
};
use std::borrow::Cow;

/// Re-export for convenience
pub use boring::ssl::CertificateCompressionAlgorithm as CertCompressAlg;

/// TLS protocol version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TlsVersion(pub(crate) boring::ssl::SslVersion);

impl std::hash::Hash for TlsVersion {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let id: u8 = match self.0 {
            boring::ssl::SslVersion::TLS1 => 0,
            boring::ssl::SslVersion::TLS1_1 => 1,
            boring::ssl::SslVersion::TLS1_2 => 2,
            boring::ssl::SslVersion::TLS1_3 => 3,
            _ => 255,
        };
        id.hash(state);
    }
}

impl TlsVersion {
    /// TLS 1.0
    pub const TLS_1_0: TlsVersion = TlsVersion(boring::ssl::SslVersion::TLS1);
    /// TLS 1.1
    pub const TLS_1_1: TlsVersion = TlsVersion(boring::ssl::SslVersion::TLS1_1);
    /// TLS 1.2
    pub const TLS_1_2: TlsVersion = TlsVersion(boring::ssl::SslVersion::TLS1_2);
    /// TLS 1.3
    pub const TLS_1_3: TlsVersion = TlsVersion(boring::ssl::SslVersion::TLS1_3);
}

/// TLS ALPN protocol.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct AlpnProtocol(pub(crate) &'static [u8]);

impl AlpnProtocol {
    /// HTTP/1.1
    pub const HTTP1: AlpnProtocol = AlpnProtocol(b"http/1.1");
    /// HTTP/2
    pub const HTTP2: AlpnProtocol = AlpnProtocol(b"h2");
    /// HTTP/3
    pub const HTTP3: AlpnProtocol = AlpnProtocol(b"h3");

    /// Create custom ALPN protocol.
    #[inline]
    pub const fn new(value: &'static [u8]) -> Self {
        AlpnProtocol(value)
    }

    /// Encode sequence for wire format.
    pub fn encode_wire_format(protocols: &[AlpnProtocol]) -> Vec<u8> {
        let mut buf = Vec::new();
        for proto in protocols {
            buf.push(proto.0.len() as u8);
            buf.extend_from_slice(proto.0);
        }
        buf
    }
}

/// TLS ALPS protocol (Application-Layer Protocol Settings).
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct AlpsProtocol(pub(crate) &'static [u8]);

impl AlpsProtocol {
    /// HTTP/1.1
    pub const HTTP1: AlpsProtocol = AlpsProtocol(b"http/1.1");
    /// HTTP/2
    pub const HTTP2: AlpsProtocol = AlpsProtocol(b"h2");
}

/// Builder for `TlsOptions`.
#[must_use]
#[derive(Debug, Clone)]
pub struct TlsOptionsBuilder {
    config: TlsOptions,
}

impl Default for TlsOptionsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// TLS connection configuration options.
///
/// Provides fine-grained control including:
/// - **Protocol negotiation** (ALPN, ALPS, TLS versions)
/// - **Session management** (tickets, PSK, key shares)
/// - **Security & privacy** (OCSP, GREASE, ECH, delegated credentials)
/// - **Performance tuning** (record size, cipher preferences)
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct TlsOptions {
    // === Protocol Negotiation ===
    /// ALPN protocols (Application-Layer Protocol Negotiation).
    pub alpn_protocols: Option<Cow<'static, [AlpnProtocol]>>,
    /// ALPS protocols (Application-Layer Protocol Settings).
    pub alps_protocols: Option<Cow<'static, [AlpsProtocol]>>,
    /// Use alternative ALPS codepoint.
    pub alps_use_new_codepoint: bool,
    /// Minimum TLS version.
    pub min_tls_version: Option<TlsVersion>,
    /// Maximum TLS version.
    pub max_tls_version: Option<TlsVersion>,

    // === Session Management ===
    /// Enable session tickets (RFC 5077).
    pub session_ticket: bool,
    /// Enable Pre-Shared Key cipher suites.
    pub pre_shared_key: bool,
    /// Skip session tickets when using PSK.
    pub psk_skip_session_ticket: bool,
    /// Enable PSK with (EC)DHE key establishment.
    pub psk_dhe_ke: bool,
    /// Maximum key shares in ClientHello.
    pub key_shares_limit: Option<u8>,

    // === Security & Privacy ===
    /// Enable OCSP stapling.
    pub enable_ocsp_stapling: bool,
    /// Enable Signed Certificate Timestamps.
    pub enable_signed_cert_timestamps: bool,
    /// Enable GREASE ECH extension.
    pub enable_ech_grease: bool,
    /// Enable GREASE extensions (RFC 8701).
    pub grease_enabled: Option<bool>,
    /// Permute ClientHello extensions.
    pub permute_extensions: Option<bool>,
    /// Enable TLS renegotiation.
    pub renegotiation: bool,
    /// Delegated credentials (RFC 9345).
    pub delegated_credentials: Option<Cow<'static, str>>,

    // === Cipher Configuration ===
    /// Cipher suite configuration string.
    pub cipher_list: Option<Cow<'static, str>>,
    /// Supported elliptic curves.
    pub curves_list: Option<Cow<'static, str>>,
    /// Supported signature algorithms.
    pub sigalgs_list: Option<Cow<'static, str>>,
    /// Certificate compression algorithms (RFC 8879).
    pub certificate_compression_algorithms: Option<Cow<'static, [CertificateCompressionAlgorithm]>>,

    // === Extension Configuration ===
    /// Extension permutation order.
    pub extension_permutation: Option<Cow<'static, [ExtensionType]>>,
    /// Maximum TLS record size.
    pub record_size_limit: Option<u16>,

    // === Hardware Overrides ===
    /// Override AES hardware acceleration.
    pub aes_hw_override: Option<bool>,
    /// Preserve TLS 1.3 cipher list order.
    pub preserve_tls13_cipher_list: Option<bool>,
}

impl Default for TlsOptions {
    fn default() -> Self {
        Self {
            alpn_protocols: Some(Cow::Borrowed(&[AlpnProtocol::HTTP2, AlpnProtocol::HTTP1])),
            alps_protocols: None,
            alps_use_new_codepoint: false,
            min_tls_version: Some(TlsVersion::TLS_1_2),
            max_tls_version: Some(TlsVersion::TLS_1_3),
            session_ticket: true,
            pre_shared_key: false,
            psk_skip_session_ticket: false,
            psk_dhe_ke: true,
            key_shares_limit: None,
            enable_ocsp_stapling: false,
            enable_signed_cert_timestamps: false,
            enable_ech_grease: false,
            grease_enabled: None,
            permute_extensions: None,
            renegotiation: true,
            delegated_credentials: None,
            cipher_list: None,
            curves_list: None,
            sigalgs_list: None,
            certificate_compression_algorithms: None,
            extension_permutation: None,
            record_size_limit: None,
            aes_hw_override: None,
            preserve_tls13_cipher_list: None,
        }
    }
}

impl TlsOptions {
    /// Create a new builder.
    #[inline]
    pub fn builder() -> TlsOptionsBuilder {
        TlsOptionsBuilder::new()
    }

    /// Apply these options to a BoringSSL connector builder.
    pub fn apply_to_builder(&self, builder: &mut SslConnectorBuilder) -> Result<(), NetError> {
        // Verification mode
        builder.set_verify(SslVerifyMode::PEER);

        // TLS versions
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

        // ALPN
        if let Some(ref alpn) = self.alpn_protocols {
            let wire = AlpnProtocol::encode_wire_format(alpn);
            builder
                .set_alpn_protos(&wire)
                .map_err(|_| NetError::SslProtocolError)?;
        }

        // Cipher configuration
        if let Some(ref ciphers) = self.cipher_list {
            builder
                .set_cipher_list(ciphers)
                .map_err(|_| NetError::SslProtocolError)?;
        }
        if let Some(ref curves) = self.curves_list {
            builder
                .set_curves_list(curves)
                .map_err(|_| NetError::SslProtocolError)?;
        }
        if let Some(ref sigalgs) = self.sigalgs_list {
            builder
                .set_sigalgs_list(sigalgs)
                .map_err(|_| NetError::SslProtocolError)?;
        }

        // GREASE
        if let Some(grease) = self.grease_enabled {
            builder.set_grease_enabled(grease);
        }

        // Permute extensions
        if let Some(permute) = self.permute_extensions {
            builder.set_permute_extensions(permute);
        }

        // Certificate compression - BoringSSL 4.x requires CertificateCompressor trait
        // TODO: Implement custom compressor if needed
        // if let Some(ref algs) = self.certificate_compression_algorithms { ... }

        Ok(())
    }
}

// === TlsOptionsBuilder Implementation ===

impl TlsOptionsBuilder {
    /// Create new builder with defaults.
    pub fn new() -> Self {
        Self {
            config: TlsOptions::default(),
        }
    }

    /// Set ALPN protocols.
    #[inline]
    pub fn alpn_protocols<I>(mut self, alpn: I) -> Self
    where
        I: IntoIterator<Item = AlpnProtocol>,
    {
        self.config.alpn_protocols = Some(Cow::Owned(alpn.into_iter().collect()));
        self
    }

    /// Set ALPS protocols.
    #[inline]
    pub fn alps_protocols<I>(mut self, alps: I) -> Self
    where
        I: IntoIterator<Item = AlpsProtocol>,
    {
        self.config.alps_protocols = Some(Cow::Owned(alps.into_iter().collect()));
        self
    }

    /// Set ALPS new codepoint flag.
    #[inline]
    pub fn alps_use_new_codepoint(mut self, enabled: bool) -> Self {
        self.config.alps_use_new_codepoint = enabled;
        self
    }

    /// Set minimum TLS version.
    #[inline]
    pub fn min_tls_version<T: Into<Option<TlsVersion>>>(mut self, version: T) -> Self {
        self.config.min_tls_version = version.into();
        self
    }

    /// Set maximum TLS version.
    #[inline]
    pub fn max_tls_version<T: Into<Option<TlsVersion>>>(mut self, version: T) -> Self {
        self.config.max_tls_version = version.into();
        self
    }

    /// Set session ticket flag.
    #[inline]
    pub fn session_ticket(mut self, enabled: bool) -> Self {
        self.config.session_ticket = enabled;
        self
    }

    /// Set pre-shared key flag.
    #[inline]
    pub fn pre_shared_key(mut self, enabled: bool) -> Self {
        self.config.pre_shared_key = enabled;
        self
    }

    /// Set PSK skip session ticket flag.
    #[inline]
    pub fn psk_skip_session_ticket(mut self, skip: bool) -> Self {
        self.config.psk_skip_session_ticket = skip;
        self
    }

    /// Set PSK DHE key establishment flag.
    #[inline]
    pub fn psk_dhe_ke(mut self, enabled: bool) -> Self {
        self.config.psk_dhe_ke = enabled;
        self
    }

    /// Set key shares limit.
    #[inline]
    pub fn key_shares_limit<T: Into<Option<u8>>>(mut self, limit: T) -> Self {
        self.config.key_shares_limit = limit.into();
        self
    }

    /// Set OCSP stapling flag.
    #[inline]
    pub fn enable_ocsp_stapling(mut self, enabled: bool) -> Self {
        self.config.enable_ocsp_stapling = enabled;
        self
    }

    /// Set signed certificate timestamps flag.
    #[inline]
    pub fn enable_signed_cert_timestamps(mut self, enabled: bool) -> Self {
        self.config.enable_signed_cert_timestamps = enabled;
        self
    }

    /// Set ECH GREASE flag.
    #[inline]
    pub fn enable_ech_grease(mut self, enabled: bool) -> Self {
        self.config.enable_ech_grease = enabled;
        self
    }

    /// Set GREASE enabled flag.
    #[inline]
    pub fn grease_enabled<T: Into<Option<bool>>>(mut self, enabled: T) -> Self {
        self.config.grease_enabled = enabled.into();
        self
    }

    /// Set permute extensions flag.
    #[inline]
    pub fn permute_extensions<T: Into<Option<bool>>>(mut self, permute: T) -> Self {
        self.config.permute_extensions = permute.into();
        self
    }

    /// Set renegotiation flag.
    #[inline]
    pub fn renegotiation(mut self, enabled: bool) -> Self {
        self.config.renegotiation = enabled;
        self
    }

    /// Set delegated credentials.
    #[inline]
    pub fn delegated_credentials<T: Into<Cow<'static, str>>>(mut self, creds: T) -> Self {
        self.config.delegated_credentials = Some(creds.into());
        self
    }

    /// Set cipher list.
    #[inline]
    pub fn cipher_list<T: Into<Cow<'static, str>>>(mut self, ciphers: T) -> Self {
        self.config.cipher_list = Some(ciphers.into());
        self
    }

    /// Set curves list.
    #[inline]
    pub fn curves_list<T: Into<Cow<'static, str>>>(mut self, curves: T) -> Self {
        self.config.curves_list = Some(curves.into());
        self
    }

    /// Set signature algorithms list.
    #[inline]
    pub fn sigalgs_list<T: Into<Cow<'static, str>>>(mut self, sigalgs: T) -> Self {
        self.config.sigalgs_list = Some(sigalgs.into());
        self
    }

    /// Set certificate compression algorithms.
    #[inline]
    pub fn certificate_compression_algorithms<T>(mut self, algs: T) -> Self
    where
        T: Into<Cow<'static, [CertificateCompressionAlgorithm]>>,
    {
        self.config.certificate_compression_algorithms = Some(algs.into());
        self
    }

    /// Set extension permutation order.
    #[inline]
    pub fn extension_permutation<T>(mut self, permutation: T) -> Self
    where
        T: Into<Cow<'static, [ExtensionType]>>,
    {
        self.config.extension_permutation = Some(permutation.into());
        self
    }

    /// Set record size limit.
    #[inline]
    pub fn record_size_limit<T: Into<Option<u16>>>(mut self, limit: T) -> Self {
        self.config.record_size_limit = limit.into();
        self
    }

    /// Set AES hardware override.
    #[inline]
    pub fn aes_hw_override<T: Into<Option<bool>>>(mut self, enabled: T) -> Self {
        self.config.aes_hw_override = enabled.into();
        self
    }

    /// Set preserve TLS 1.3 cipher list flag.
    #[inline]
    pub fn preserve_tls13_cipher_list<T: Into<Option<bool>>>(mut self, enabled: T) -> Self {
        self.config.preserve_tls13_cipher_list = enabled.into();
        self
    }

    /// Build the TlsOptions.
    #[inline]
    pub fn build(self) -> TlsOptions {
        self.config
    }
}
