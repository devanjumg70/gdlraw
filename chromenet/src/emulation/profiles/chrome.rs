//! Chrome browser profiles.
//!
//! Provides emulation configurations for various Chrome versions.

use crate::emulation::{Emulation, EmulationFactory, Http2Options};
use crate::socket::tls::{AlpnProtocol, TlsOptions, TlsVersion};
use http::{header, HeaderMap, HeaderValue};

/// Chrome browser versions for emulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
#[derive(Default)]
pub enum Chrome {
    /// Chrome 100
    V100,
    /// Chrome 104
    V104,
    /// Chrome 107
    V107,
    /// Chrome 110
    V110,
    /// Chrome 114
    V114,
    /// Chrome 117
    V117,
    /// Chrome 120
    V120,
    /// Chrome 123
    V123,
    /// Chrome 124
    V124,
    /// Chrome 126
    V126,
    /// Chrome 127
    V127,
    /// Chrome 128
    V128,
    /// Chrome 129
    V129,
    /// Chrome 131
    V131,
    /// Chrome 133
    V133,
    /// Chrome 135
    V135,
    /// Chrome 137
    V137,
    /// Chrome 139
    V139,
    /// Chrome 140
    V140,
    /// Chrome 141
    V141,
    /// Chrome 143 (latest)
    #[default]
    V143,
}

impl EmulationFactory for Chrome {
    fn emulation(self) -> Emulation {
        chrome_emulation(self.version_string())
    }
}

impl Chrome {
    /// Get version string for this Chrome version.
    pub fn version_string(self) -> &'static str {
        match self {
            Chrome::V100 => "100.0.0.0",
            Chrome::V104 => "104.0.0.0",
            Chrome::V107 => "107.0.0.0",
            Chrome::V110 => "110.0.0.0",
            Chrome::V114 => "114.0.0.0",
            Chrome::V117 => "117.0.0.0",
            Chrome::V120 => "120.0.0.0",
            Chrome::V123 => "123.0.0.0",
            Chrome::V124 => "124.0.0.0",
            Chrome::V126 => "126.0.0.0",
            Chrome::V127 => "127.0.0.0",
            Chrome::V128 => "128.0.0.0",
            Chrome::V129 => "129.0.0.0",
            Chrome::V131 => "131.0.0.0",
            Chrome::V133 => "133.0.0.0",
            Chrome::V135 => "135.0.0.0",
            Chrome::V137 => "137.0.0.0",
            Chrome::V139 => "139.0.0.0",
            Chrome::V140 => "140.0.0.0",
            Chrome::V141 => "141.0.0.0",
            Chrome::V143 => "143.0.0.0",
        }
    }

    /// Get major version number.
    pub fn major_version(self) -> u16 {
        match self {
            Chrome::V100 => 100,
            Chrome::V104 => 104,
            Chrome::V107 => 107,
            Chrome::V110 => 110,
            Chrome::V114 => 114,
            Chrome::V117 => 117,
            Chrome::V120 => 120,
            Chrome::V123 => 123,
            Chrome::V124 => 124,
            Chrome::V126 => 126,
            Chrome::V127 => 127,
            Chrome::V128 => 128,
            Chrome::V129 => 129,
            Chrome::V131 => 131,
            Chrome::V133 => 133,
            Chrome::V135 => 135,
            Chrome::V137 => 137,
            Chrome::V139 => 139,
            Chrome::V140 => 140,
            Chrome::V141 => 141,
            Chrome::V143 => 143,
        }
    }

    /// Get all supported versions.
    pub fn all_versions() -> &'static [Chrome] {
        &[
            Chrome::V100,
            Chrome::V104,
            Chrome::V107,
            Chrome::V110,
            Chrome::V114,
            Chrome::V117,
            Chrome::V120,
            Chrome::V123,
            Chrome::V124,
            Chrome::V126,
            Chrome::V127,
            Chrome::V128,
            Chrome::V129,
            Chrome::V131,
            Chrome::V133,
            Chrome::V135,
            Chrome::V137,
            Chrome::V139,
            Chrome::V140,
            Chrome::V141,
            Chrome::V143,
        ]
    }
}

/// Create Chrome emulation for a specific version.
fn chrome_emulation(version: &'static str) -> Emulation {
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
    let major = version.split('.').next().unwrap_or("143");

    let ua = format!(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{} Safari/537.36",
        version
    );

    if let Ok(val) = HeaderValue::from_str(&ua) {
        headers.insert(header::USER_AGENT, val);
    }
    headers.insert(
        header::ACCEPT,
        HeaderValue::from_static(
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8",
        ),
    );
    headers.insert(
        header::ACCEPT_LANGUAGE,
        HeaderValue::from_static("en-US,en;q=0.9"),
    );
    headers.insert(
        header::ACCEPT_ENCODING,
        HeaderValue::from_static("gzip, deflate, br, zstd"),
    );
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("max-age=0"));
    headers.insert(
        header::UPGRADE_INSECURE_REQUESTS,
        HeaderValue::from_static("1"),
    );

    // Sec-CH-UA headers
    if let Ok(val) = HeaderValue::from_str(&format!(
        "\"Chromium\";v=\"{}\", \"Google Chrome\";v=\"{}\", \"Not-A.Brand\";v=\"99\"",
        major, major
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
