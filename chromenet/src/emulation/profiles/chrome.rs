//! Chrome browser profiles.
//!
//! Provides emulation configurations for various Chrome versions.

use crate::emulation::{Emulation, EmulationFactory, Http2Options};
use crate::http::h2fingerprint::{H2Fingerprint, H2FingerprintBuilder};
use crate::socket::tls::{AlpnProtocol, TlsOptions, TlsVersion};
use http::{header, HeaderMap, HeaderValue};

/// Chrome browser versions for emulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Chrome {
    /// Chrome 120
    V120,
    /// Chrome 124
    V124,
    /// Chrome 128
    V128,
    /// Chrome 131
    V131,
    /// Chrome 135
    V135,
    /// Chrome 140 (latest)
    V140,
}

impl Default for Chrome {
    fn default() -> Self {
        Chrome::V140
    }
}

impl EmulationFactory for Chrome {
    fn emulation(self) -> Emulation {
        match self {
            Chrome::V120 => chrome_v120(),
            Chrome::V124 => chrome_v124(),
            Chrome::V128 => chrome_v128(),
            Chrome::V131 => chrome_v131(),
            Chrome::V135 => chrome_v135(),
            Chrome::V140 => chrome_v140(),
        }
    }
}

/// Create Chrome v120 emulation.
pub fn chrome_v120() -> Emulation {
    chrome_emulation("120.0.0.0")
}

/// Create Chrome v124 emulation.
pub fn chrome_v124() -> Emulation {
    chrome_emulation("124.0.0.0")
}

/// Create Chrome v128 emulation.
pub fn chrome_v128() -> Emulation {
    chrome_emulation("128.0.0.0")
}

/// Create Chrome v131 emulation.
pub fn chrome_v131() -> Emulation {
    chrome_emulation("131.0.0.0")
}

/// Create Chrome v135 emulation.
pub fn chrome_v135() -> Emulation {
    chrome_emulation("135.0.0.0")
}

/// Create Chrome v140 emulation.
pub fn chrome_v140() -> Emulation {
    chrome_emulation("140.0.0.0")
}

/// Create Chrome emulation for a specific version.
fn chrome_emulation(version: &str) -> Emulation {
    let tls = chrome_tls_options();
    let h2 = chrome_h2_options();
    let headers = chrome_headers(version);

    Emulation::builder()
        .tls_options(tls)
        .http2_options(h2)
        .headers(headers)
        .build()
}

/// Chrome TLS configuration.
fn chrome_tls_options() -> TlsOptions {
    TlsOptions::builder()
        .alpn_protocols([AlpnProtocol::HTTP2, AlpnProtocol::HTTP1])
        .min_tls_version(TlsVersion::TLS_1_2)
        .max_tls_version(TlsVersion::TLS_1_3)
        .cipher_list(
            "TLS_AES_128_GCM_SHA256:TLS_AES_256_GCM_SHA384:TLS_CHACHA20_POLY1305_SHA256:\
             ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:\
             ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384:\
             ECDHE-ECDSA-CHACHA20-POLY1305:ECDHE-RSA-CHACHA20-POLY1305",
        )
        .curves_list("X25519:P-256:P-384")
        .sigalgs_list(
            "ecdsa_secp256r1_sha256:rsa_pss_rsae_sha256:rsa_pkcs1_sha256:\
             ecdsa_secp384r1_sha384:rsa_pss_rsae_sha384:rsa_pkcs1_sha384:\
             rsa_pss_rsae_sha512:rsa_pkcs1_sha512",
        )
        .grease_enabled(true)
        .permute_extensions(true)
        .enable_ocsp_stapling(true)
        .enable_signed_cert_timestamps(true)
        .session_ticket(true)
        .build()
}

/// Chrome HTTP/2 configuration.
fn chrome_h2_options() -> Http2Options {
    Http2Options::builder()
        .initial_window_size(6291456)
        .max_header_list_size(262144)
        .header_table_size(65536)
        .enable_push(false)
        .build()
}

/// Chrome default headers.
fn chrome_headers(version: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();

    let ua = format!(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{} Safari/537.36",
        version
    );

    if let Ok(val) = HeaderValue::from_str(&ua) {
        headers.insert(header::USER_AGENT, val);
    }
    headers.insert(header::ACCEPT, HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8"));
    headers.insert(
        header::ACCEPT_LANGUAGE,
        HeaderValue::from_static("en-US,en;q=0.9"),
    );
    headers.insert(
        header::ACCEPT_ENCODING,
        HeaderValue::from_static("gzip, deflate, br"),
    );
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("max-age=0"));
    headers.insert(
        header::UPGRADE_INSECURE_REQUESTS,
        HeaderValue::from_static("1"),
    );

    // Sec-CH-UA headers
    if let Ok(val) = HeaderValue::from_str(&format!(
        "\"Chromium\";v=\"{}\", \"Google Chrome\";v=\"{}\"",
        version.split('.').next().unwrap_or("140"),
        version.split('.').next().unwrap_or("140")
    )) {
        headers.insert("sec-ch-ua", val);
    }
    headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?0"));
    headers.insert(
        "sec-ch-ua-platform",
        HeaderValue::from_static("\"Windows\""),
    );
    headers.insert("sec-fetch-dest", HeaderValue::from_static("document"));
    headers.insert("sec-fetch-mode", HeaderValue::from_static("navigate"));
    headers.insert("sec-fetch-site", HeaderValue::from_static("none"));
    headers.insert("sec-fetch-user", HeaderValue::from_static("?1"));

    headers
}
