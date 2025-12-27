//! Firefox browser profiles.
//!
//! Provides emulation configurations for various Firefox versions.

use crate::emulation::{Emulation, EmulationFactory, Http2Options};
use crate::socket::tls::{AlpnProtocol, TlsOptions, TlsVersion};
use http::{header, HeaderMap, HeaderValue};

/// Firefox browser versions for emulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Firefox {
    /// Firefox 128
    V128,
    /// Firefox 133
    V133,
    /// Firefox 135
    V135,
    /// Firefox 140
    V140,
    /// Firefox 145 (latest)
    V145,
}

impl Default for Firefox {
    fn default() -> Self {
        Firefox::V145
    }
}

impl EmulationFactory for Firefox {
    fn emulation(self) -> Emulation {
        match self {
            Firefox::V128 => firefox_v128(),
            Firefox::V133 => firefox_v133(),
            Firefox::V135 => firefox_v135(),
            Firefox::V140 => firefox_v140(),
            Firefox::V145 => firefox_v145(),
        }
    }
}

/// Create Firefox v128 emulation.
pub fn firefox_v128() -> Emulation {
    firefox_emulation("128.0")
}

/// Create Firefox v133 emulation.
pub fn firefox_v133() -> Emulation {
    firefox_emulation("133.0")
}

/// Create Firefox v135 emulation.
pub fn firefox_v135() -> Emulation {
    firefox_emulation("135.0")
}

/// Create Firefox v140 emulation.
pub fn firefox_v140() -> Emulation {
    firefox_emulation("140.0")
}

/// Create Firefox v145 emulation.
pub fn firefox_v145() -> Emulation {
    firefox_emulation("145.0")
}

/// Create Firefox emulation for a specific version.
fn firefox_emulation(version: &str) -> Emulation {
    let tls = firefox_tls_options();
    let h2 = firefox_h2_options();
    let headers = firefox_headers(version);

    Emulation::builder()
        .tls_options(tls)
        .http2_options(h2)
        .headers(headers)
        .build()
}

/// Firefox TLS configuration.
/// Firefox uses different cipher order and curves than Chrome.
fn firefox_tls_options() -> TlsOptions {
    TlsOptions::builder()
        .alpn_protocols([AlpnProtocol::HTTP2, AlpnProtocol::HTTP1])
        .min_tls_version(TlsVersion::TLS_1_2)
        .max_tls_version(TlsVersion::TLS_1_3)
        // Firefox cipher order differs from Chrome
        .cipher_list(
            "TLS_AES_128_GCM_SHA256:TLS_CHACHA20_POLY1305_SHA256:TLS_AES_256_GCM_SHA384:\
             ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:\
             ECDHE-ECDSA-CHACHA20-POLY1305:ECDHE-RSA-CHACHA20-POLY1305:\
             ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384:\
             ECDHE-ECDSA-AES256-SHA:ECDHE-ECDSA-AES128-SHA:\
             ECDHE-RSA-AES128-SHA:ECDHE-RSA-AES256-SHA:\
             AES128-GCM-SHA256:AES256-GCM-SHA384:AES128-SHA:AES256-SHA",
        )
        // Firefox curve order
        .curves_list("X25519:P-256:P-384:P-521")
        .sigalgs_list(
            "ecdsa_secp256r1_sha256:ecdsa_secp384r1_sha384:ecdsa_secp521r1_sha512:\
             rsa_pss_rsae_sha256:rsa_pss_rsae_sha384:rsa_pss_rsae_sha512:\
             rsa_pkcs1_sha256:rsa_pkcs1_sha384:rsa_pkcs1_sha512",
        )
        .grease_enabled(false) // Firefox doesn't use GREASE
        .permute_extensions(false)
        .enable_ocsp_stapling(true)
        .enable_signed_cert_timestamps(true)
        .session_ticket(true)
        .build()
}

/// Firefox HTTP/2 configuration.
/// Firefox uses different SETTINGS than Chrome.
fn firefox_h2_options() -> Http2Options {
    Http2Options::builder()
        .initial_window_size(131072) // Firefox: 131072 (smaller than Chrome's 6MB)
        .max_header_list_size(65536)
        .header_table_size(65536)
        .enable_push(true) // Firefox enables push by default
        .build()
}

/// Firefox default headers.
fn firefox_headers(version: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();

    let ua = format!(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:{}) Gecko/20100101 Firefox/{}",
        version, version
    );

    if let Ok(val) = HeaderValue::from_str(&ua) {
        headers.insert(header::USER_AGENT, val);
    }
    headers.insert(
        header::ACCEPT,
        HeaderValue::from_static(
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8",
        ),
    );
    headers.insert(
        header::ACCEPT_LANGUAGE,
        HeaderValue::from_static("en-US,en;q=0.5"),
    );
    headers.insert(
        header::ACCEPT_ENCODING,
        HeaderValue::from_static("gzip, deflate, br"),
    );
    headers.insert(header::CONNECTION, HeaderValue::from_static("keep-alive"));
    headers.insert(
        header::UPGRADE_INSECURE_REQUESTS,
        HeaderValue::from_static("1"),
    );

    // Firefox-specific headers (no sec-ch-ua, different sec-fetch)
    headers.insert("sec-fetch-dest", HeaderValue::from_static("document"));
    headers.insert("sec-fetch-mode", HeaderValue::from_static("navigate"));
    headers.insert("sec-fetch-site", HeaderValue::from_static("none"));
    headers.insert("sec-fetch-user", HeaderValue::from_static("?1"));
    // Firefox sends Priority header
    headers.insert("priority", HeaderValue::from_static("u=1"));

    headers
}
