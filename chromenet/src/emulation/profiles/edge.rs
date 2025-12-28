//! Edge browser profiles.
//!
//! Edge is Chromium-based, so TLS fingerprint is similar to Chrome.

use crate::emulation::{Emulation, EmulationFactory, Http2Options};
use crate::socket::tls::{AlpnProtocol, TlsOptions, TlsVersion};
use http::{header, HeaderMap, HeaderValue};

/// Edge browser versions for emulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
#[derive(Default)]
pub enum Edge {
    /// Edge 101
    V101,
    /// Edge 120
    V120,
    /// Edge 127
    V127,
    /// Edge 131
    V131,
    /// Edge 134
    V134,
    /// Edge 135
    V135,
    /// Edge 137
    V137,
    /// Edge 139
    V139,
    /// Edge 141
    V141,
    /// Edge 142 (latest)
    #[default]
    V142,
}

impl EmulationFactory for Edge {
    fn emulation(self) -> Emulation {
        edge_emulation(self.version_string())
    }
}

impl Edge {
    /// Get version string.
    pub fn version_string(self) -> &'static str {
        match self {
            Edge::V101 => "101.0.0.0",
            Edge::V120 => "120.0.0.0",
            Edge::V127 => "127.0.0.0",
            Edge::V131 => "131.0.0.0",
            Edge::V134 => "134.0.0.0",
            Edge::V135 => "135.0.0.0",
            Edge::V137 => "137.0.0.0",
            Edge::V139 => "139.0.0.0",
            Edge::V141 => "141.0.0.0",
            Edge::V142 => "142.0.0.0",
        }
    }

    /// Get all supported versions.
    pub fn all_versions() -> &'static [Edge] {
        &[
            Edge::V101,
            Edge::V120,
            Edge::V127,
            Edge::V131,
            Edge::V134,
            Edge::V135,
            Edge::V137,
            Edge::V139,
            Edge::V141,
            Edge::V142,
        ]
    }
}

/// Create Edge emulation for a specific version.
fn edge_emulation(version: &'static str) -> Emulation {
    let tls = edge_tls_options();
    let h2 = edge_h2_options();
    let headers = edge_headers(version);

    Emulation::builder()
        .tls_options(tls)
        .http2_options(h2)
        .headers(headers)
        .build()
}

/// Edge TLS configuration (Chromium-based).
fn edge_tls_options() -> TlsOptions {
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
        .build()
}

/// Edge HTTP/2 configuration.
fn edge_h2_options() -> Http2Options {
    Http2Options::builder()
        .initial_window_size(6291456)
        .max_header_list_size(262144)
        .header_table_size(65536)
        .enable_push(false)
        .build()
}

/// Edge default headers.
fn edge_headers(version: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let major = version.split('.').next().unwrap_or("142");

    let ua = format!(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{} Safari/537.36 Edg/{}",
        version, version
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

    // Edge sec-ch-ua headers
    if let Ok(val) = HeaderValue::from_str(&format!(
        "\"Microsoft Edge\";v=\"{}\", \"Chromium\";v=\"{}\", \"Not-A.Brand\";v=\"99\"",
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
