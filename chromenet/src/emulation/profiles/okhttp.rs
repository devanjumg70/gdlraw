//! OkHttp client profiles.
//!
//! Provides emulation configurations for OkHttp Android HTTP client.

use crate::emulation::{Emulation, EmulationFactory, Http2Options};
use crate::socket::tls::{AlpnProtocol, TlsOptions, TlsVersion};
use http::{header, HeaderMap, HeaderValue};

/// OkHttp versions for emulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum OkHttp {
    /// OkHttp 3.9
    V3_9,
    /// OkHttp 3.11
    V3_11,
    /// OkHttp 3.13
    V3_13,
    /// OkHttp 3.14
    V3_14,
    /// OkHttp 4.9
    V4_9,
    /// OkHttp 4.10
    V4_10,
    /// OkHttp 4.12
    V4_12,
    /// OkHttp 5.0
    V5,
}

impl Default for OkHttp {
    fn default() -> Self {
        OkHttp::V5
    }
}

impl EmulationFactory for OkHttp {
    fn emulation(self) -> Emulation {
        match self {
            OkHttp::V3_9 => okhttp_v3_9(),
            OkHttp::V3_11 => okhttp_v3_11(),
            OkHttp::V3_13 => okhttp_v3_13(),
            OkHttp::V3_14 => okhttp_v3_14(),
            OkHttp::V4_9 => okhttp_v4_9(),
            OkHttp::V4_10 => okhttp_v4_10(),
            OkHttp::V4_12 => okhttp_v4_12(),
            OkHttp::V5 => okhttp_v5(),
        }
    }
}

// Common constants
const CURVES: &str = "X25519:P-256:P-384";

const SIGALGS: &str = "ecdsa_secp256r1_sha256:rsa_pss_rsae_sha256:rsa_pkcs1_sha256:\
    ecdsa_secp384r1_sha384:rsa_pss_rsae_sha384:rsa_pkcs1_sha384:\
    rsa_pss_rsae_sha512:rsa_pkcs1_sha512:rsa_pkcs1_sha1";

// OkHttp 3.x cipher list (TLS 1.2 only)
const OKHTTP3_CIPHERS: &str = "ECDHE-ECDSA-AES128-GCM-SHA256:\
    ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:\
    ECDHE-RSA-AES256-GCM-SHA384:ECDHE-ECDSA-CHACHA20-POLY1305:\
    ECDHE-RSA-CHACHA20-POLY1305:ECDHE-ECDSA-AES128-SHA:\
    ECDHE-RSA-AES128-SHA:ECDHE-ECDSA-AES256-SHA:ECDHE-RSA-AES256-SHA:\
    AES128-GCM-SHA256:AES256-GCM-SHA384:AES128-SHA:AES256-SHA:DES-CBC3-SHA";

// OkHttp 4.x/5.x cipher list (TLS 1.3 support)
const OKHTTP4_CIPHERS: &str = "TLS_AES_128_GCM_SHA256:TLS_AES_256_GCM_SHA384:\
    TLS_CHACHA20_POLY1305_SHA256:ECDHE-ECDSA-AES128-GCM-SHA256:\
    ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:\
    ECDHE-RSA-AES256-GCM-SHA384:ECDHE-ECDSA-CHACHA20-POLY1305:\
    ECDHE-RSA-CHACHA20-POLY1305:ECDHE-RSA-AES128-SHA:ECDHE-RSA-AES256-SHA:\
    AES128-GCM-SHA256:AES256-GCM-SHA384:AES128-SHA:AES256-SHA:DES-CBC3-SHA";

/// Create OkHttp 3.9 emulation.
pub fn okhttp_v3_9() -> Emulation {
    okhttp_emulation(OKHTTP3_CIPHERS, TlsVersion::TLS_1_2, "okhttp/3.9.0")
}

/// Create OkHttp 3.11 emulation.
pub fn okhttp_v3_11() -> Emulation {
    okhttp_emulation(OKHTTP3_CIPHERS, TlsVersion::TLS_1_2, "okhttp/3.11.0")
}

/// Create OkHttp 3.13 emulation.
pub fn okhttp_v3_13() -> Emulation {
    okhttp_emulation(OKHTTP3_CIPHERS, TlsVersion::TLS_1_3, "okhttp/3.13.0")
}

/// Create OkHttp 3.14 emulation.
pub fn okhttp_v3_14() -> Emulation {
    okhttp_emulation(OKHTTP4_CIPHERS, TlsVersion::TLS_1_3, "okhttp/3.14.0")
}

/// Create OkHttp 4.9 emulation.
pub fn okhttp_v4_9() -> Emulation {
    okhttp_emulation(OKHTTP4_CIPHERS, TlsVersion::TLS_1_3, "okhttp/4.9.0")
}

/// Create OkHttp 4.10 emulation.
pub fn okhttp_v4_10() -> Emulation {
    okhttp_emulation(OKHTTP4_CIPHERS, TlsVersion::TLS_1_3, "okhttp/4.10.0")
}

/// Create OkHttp 4.12 emulation.
pub fn okhttp_v4_12() -> Emulation {
    okhttp_emulation(OKHTTP4_CIPHERS, TlsVersion::TLS_1_3, "okhttp/4.12.0")
}

/// Create OkHttp 5.0 emulation.
pub fn okhttp_v5() -> Emulation {
    okhttp_emulation(OKHTTP4_CIPHERS, TlsVersion::TLS_1_3, "okhttp/5.0.0-alpha2")
}

/// Create OkHttp emulation with specific config.
fn okhttp_emulation(cipher_list: &'static str, max_tls: TlsVersion, ua: &'static str) -> Emulation {
    let tls = TlsOptions::builder()
        .alpn_protocols([AlpnProtocol::HTTP2, AlpnProtocol::HTTP1])
        .min_tls_version(TlsVersion::TLS_1_2)
        .max_tls_version(max_tls)
        .cipher_list(cipher_list)
        .curves_list(CURVES)
        .sigalgs_list(SIGALGS)
        .enable_ocsp_stapling(true)
        // OkHttp doesn't use GREASE or extension permutation
        .grease_enabled(false)
        .permute_extensions(false)
        .build();

    let h2 = Http2Options::builder()
        .initial_window_size(6291456)
        .max_header_list_size(262144)
        .header_table_size(65536)
        .max_concurrent_streams(1000)
        .enable_push(false)
        .build();

    let mut headers = HeaderMap::new();
    if let Ok(val) = HeaderValue::from_str(ua) {
        headers.insert(header::USER_AGENT, val);
    }
    headers.insert(header::ACCEPT, HeaderValue::from_static("*/*"));
    headers.insert(
        header::ACCEPT_LANGUAGE,
        HeaderValue::from_static("en-US,en;q=0.9"),
    );
    headers.insert(
        header::ACCEPT_ENCODING,
        HeaderValue::from_static("gzip, deflate, br"),
    );

    Emulation::builder()
        .tls_options(tls)
        .http2_options(h2)
        .headers(headers)
        .build()
}
