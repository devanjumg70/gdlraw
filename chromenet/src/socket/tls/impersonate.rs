use super::{AlpnProtocol, TlsOptions, TlsVersion};
use boring::ssl::CertificateCompressionAlgorithm;

/// Browser impersonation targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImpersonateTarget {
    Chrome124,
    Chrome128,
    Firefox128,
    Firefox129,
    Safari17,
    Safari18,
    OkHttp4,
    OkHttp5,
}

impl ImpersonateTarget {
    /// Create the TLS options for this target.
    pub fn create_tls_options(&self) -> TlsOptions {
        match self {
            Self::Chrome124 | Self::Chrome128 => chrome_v124_options(),
            Self::Firefox128 | Self::Firefox129 => firefox_v128_options(),
            Self::Safari17 | Self::Safari18 => safari_v17_options(),
            Self::OkHttp4 | Self::OkHttp5 => okhttp_options(),
        }
    }
}

// --- Constants ---

// Chrome (v124+)
const CHROME_CIPHERS: &str = "TLS_AES_128_GCM_SHA256:TLS_AES_256_GCM_SHA384:TLS_CHACHA20_POLY1305_SHA256:TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256:TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256:TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384:TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384:TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256:TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256:TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA:TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA:TLS_RSA_WITH_AES_128_GCM_SHA256:TLS_RSA_WITH_AES_256_GCM_SHA384:TLS_RSA_WITH_AES_128_CBC_SHA:TLS_RSA_WITH_AES_256_CBC_SHA";
const CHROME_CURVES: &str = "X25519:P-256:P-384";
const CHROME_SIGALGS: &str = "ecdsa_secp256r1_sha256:rsa_pss_rsae_sha256:rsa_pkcs1_sha256:ecdsa_secp384r1_sha384:rsa_pss_rsae_sha384:rsa_pkcs1_sha384:rsa_pss_rsae_sha512:rsa_pkcs1_sha512";

// Firefox (v128+)
const FIREFOX_CIPHERS: &str = "TLS_AES_128_GCM_SHA256:TLS_CHACHA20_POLY1305_SHA256:TLS_AES_256_GCM_SHA384:TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256:TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256:TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256:TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256:TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384:TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384:TLS_ECDHE_ECDSA_WITH_AES_256_CBC_SHA:TLS_ECDHE_ECDSA_WITH_AES_128_CBC_SHA:TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA:TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA:TLS_RSA_WITH_AES_128_GCM_SHA256:TLS_RSA_WITH_AES_256_GCM_SHA384:TLS_RSA_WITH_AES_128_CBC_SHA:TLS_RSA_WITH_AES_256_CBC_SHA";
const FIREFOX_CURVES: &str = "X25519:P-256:P-384:P-521:ffdhe2048:ffdhe3072";
const FIREFOX_SIGALGS: &str = "ecdsa_secp256r1_sha256:ecdsa_secp384r1_sha384:ecdsa_secp521r1_sha512:rsa_pss_rsae_sha256:rsa_pss_rsae_sha384:rsa_pss_rsae_sha512:rsa_pkcs1_sha256:rsa_pkcs1_sha384:rsa_pkcs1_sha512:ecdsa_sha1:rsa_pkcs1_sha1";

// Safari (v17+)
const SAFARI_CIPHERS: &str = "TLS_AES_128_GCM_SHA256:TLS_AES_256_GCM_SHA384:TLS_CHACHA20_POLY1305_SHA256:TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384:TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256:TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256:TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384:TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256:TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256:TLS_ECDHE_ECDSA_WITH_AES_256_CBC_SHA384:TLS_ECDHE_ECDSA_WITH_AES_128_CBC_SHA256:TLS_ECDHE_ECDSA_WITH_AES_256_CBC_SHA:TLS_ECDHE_ECDSA_WITH_AES_128_CBC_SHA:TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA384:TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA256:TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA:TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA:TLS_RSA_WITH_AES_256_GCM_SHA384:TLS_RSA_WITH_AES_128_GCM_SHA256:TLS_RSA_WITH_AES_256_CBC_SHA256:TLS_RSA_WITH_AES_128_CBC_SHA256:TLS_RSA_WITH_AES_256_CBC_SHA:TLS_RSA_WITH_AES_128_CBC_SHA:TLS_ECDHE_ECDSA_WITH_3DES_EDE_CBC_SHA:TLS_ECDHE_RSA_WITH_3DES_EDE_CBC_SHA:TLS_RSA_WITH_3DES_EDE_CBC_SHA";
const SAFARI_CURVES: &str = "X25519:P-256:P-384:P-521";
const SAFARI_SIGALGS: &str = "ecdsa_secp256r1_sha256:rsa_pss_rsae_sha256:rsa_pkcs1_sha256:ecdsa_secp384r1_sha384:ecdsa_sha1:rsa_pss_rsae_sha384:rsa_pss_rsae_sha384:rsa_pkcs1_sha384:rsa_pss_rsae_sha512:rsa_pkcs1_sha512:rsa_pkcs1_sha1";

// OkHttp (Generic)
const OKHTTP_CIPHERS: &str = "TLS_AES_128_GCM_SHA256:TLS_AES_256_GCM_SHA384:TLS_CHACHA20_POLY1305_SHA256:TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256:TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256:TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384:TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384:TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256:TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256:TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA:TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA:TLS_RSA_WITH_AES_128_GCM_SHA256:TLS_RSA_WITH_AES_256_GCM_SHA384:TLS_RSA_WITH_AES_128_CBC_SHA:TLS_RSA_WITH_AES_256_CBC_SHA";
const OKHTTP_CURVES: &str = "X25519:P-256:P-384";

fn chrome_v124_options() -> TlsOptions {
    TlsOptions::builder()
        .cipher_list(CHROME_CIPHERS)
        .curves_list(CHROME_CURVES)
        .sigalgs_list(CHROME_SIGALGS)
        .min_tls_version(TlsVersion::TLS_1_2)
        .max_tls_version(TlsVersion::TLS_1_3)
        .enable_ech_grease(true)
        .grease_enabled(true)
        .permute_extensions(true)
        .pre_shared_key(true)
        .enable_ocsp_stapling(true)
        .enable_signed_cert_timestamps(true)
        .certificate_compression_algorithms(&[CertificateCompressionAlgorithm::BROTLI])
        .build()
}

fn firefox_v128_options() -> TlsOptions {
    TlsOptions::builder()
        .cipher_list(FIREFOX_CIPHERS)
        .curves_list(FIREFOX_CURVES)
        .sigalgs_list(FIREFOX_SIGALGS)
        .min_tls_version(TlsVersion::TLS_1_2)
        .max_tls_version(TlsVersion::TLS_1_3)
        .enable_ech_grease(true)
        .pre_shared_key(true)
        .enable_ocsp_stapling(true)
        .certificate_compression_algorithms(&[
            CertificateCompressionAlgorithm::ZLIB,
            CertificateCompressionAlgorithm::BROTLI,
            CertificateCompressionAlgorithm::ZSTD,
        ])
        .build()
}

fn safari_v17_options() -> TlsOptions {
    TlsOptions::builder()
        .cipher_list(SAFARI_CIPHERS)
        .curves_list(SAFARI_CURVES)
        .sigalgs_list(SAFARI_SIGALGS)
        .min_tls_version(TlsVersion::TLS_1_0)
        .max_tls_version(TlsVersion::TLS_1_3)
        .session_ticket(false)
        .grease_enabled(true)
        .enable_ocsp_stapling(true)
        .enable_signed_cert_timestamps(true)
        .certificate_compression_algorithms(&[CertificateCompressionAlgorithm::ZLIB])
        .build()
}

fn okhttp_options() -> TlsOptions {
    TlsOptions::builder()
        .cipher_list(OKHTTP_CIPHERS)
        .curves_list(OKHTTP_CURVES)
        .min_tls_version(TlsVersion::TLS_1_2)
        .max_tls_version(TlsVersion::TLS_1_3)
        .build()
}
