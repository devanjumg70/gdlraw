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
    /// Firefox 109
    V109,
    /// Firefox 117
    V117,
    /// Firefox 128
    V128,
    /// Firefox 133
    V133,
    /// Firefox 135
    V135,
    /// Firefox 136
    V136,
    /// Firefox 139
    V139,
    /// Firefox 142
    V142,
    /// Firefox 143
    V143,
    /// Firefox 144
    V144,
    /// Firefox 145 (latest)
    V145,
    /// Firefox Private Browsing mode
    Private,
    /// Firefox on Android
    Android,
}

impl Default for Firefox {
    fn default() -> Self {
        Firefox::V145
    }
}

impl EmulationFactory for Firefox {
    fn emulation(self) -> Emulation {
        firefox_emulation(self.version_string(), self.is_private(), self.is_android())
    }
}

impl Firefox {
    /// Get version string.
    pub fn version_string(self) -> &'static str {
        match self {
            Firefox::V109 => "109.0",
            Firefox::V117 => "117.0",
            Firefox::V128 => "128.0",
            Firefox::V133 => "133.0",
            Firefox::V135 => "135.0",
            Firefox::V136 => "136.0",
            Firefox::V139 => "139.0",
            Firefox::V142 => "142.0",
            Firefox::V143 => "143.0",
            Firefox::V144 => "144.0",
            Firefox::V145 => "145.0",
            Firefox::Private => "145.0",
            Firefox::Android => "145.0",
        }
    }

    /// Check if private browsing mode.
    pub fn is_private(self) -> bool {
        matches!(self, Firefox::Private)
    }

    /// Check if Android version.
    pub fn is_android(self) -> bool {
        matches!(self, Firefox::Android)
    }

    /// Get all supported versions.
    pub fn all_versions() -> &'static [Firefox] {
        &[
            Firefox::V109,
            Firefox::V117,
            Firefox::V128,
            Firefox::V133,
            Firefox::V135,
            Firefox::V136,
            Firefox::V139,
            Firefox::V142,
            Firefox::V143,
            Firefox::V144,
            Firefox::V145,
            Firefox::Private,
            Firefox::Android,
        ]
    }
}

/// Create Firefox emulation for a specific version.
fn firefox_emulation(version: &'static str, is_private: bool, is_android: bool) -> Emulation {
    let tls = firefox_tls_options();
    let h2 = firefox_h2_options();
    let headers = firefox_headers(version, is_private, is_android);

    Emulation::builder()
        .tls_options(tls)
        .http2_options(h2)
        .headers(headers)
        .build()
}

/// Firefox TLS configuration.
fn firefox_tls_options() -> TlsOptions {
    TlsOptions::builder()
        .alpn_protocols([AlpnProtocol::HTTP2, AlpnProtocol::HTTP1])
        .min_tls_version(TlsVersion::TLS_1_2)
        .max_tls_version(TlsVersion::TLS_1_3)
        .cipher_list(
            "TLS_AES_128_GCM_SHA256:TLS_CHACHA20_POLY1305_SHA256:TLS_AES_256_GCM_SHA384:\
             ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:\
             ECDHE-ECDSA-CHACHA20-POLY1305:ECDHE-RSA-CHACHA20-POLY1305:\
             ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384:\
             ECDHE-ECDSA-AES256-SHA:ECDHE-ECDSA-AES128-SHA:\
             ECDHE-RSA-AES128-SHA:ECDHE-RSA-AES256-SHA:\
             AES128-GCM-SHA256:AES256-GCM-SHA384:AES128-SHA:AES256-SHA",
        )
        .curves_list("X25519:P-256:P-384:P-521")
        .sigalgs_list(
            "ecdsa_secp256r1_sha256:ecdsa_secp384r1_sha384:ecdsa_secp521r1_sha512:\
             rsa_pss_rsae_sha256:rsa_pss_rsae_sha384:rsa_pss_rsae_sha512:\
             rsa_pkcs1_sha256:rsa_pkcs1_sha384:rsa_pkcs1_sha512",
        )
        .grease_enabled(false)
        .permute_extensions(false)
        .enable_ocsp_stapling(true)
        .enable_signed_cert_timestamps(true)
        .session_ticket(true)
        .build()
}

/// Firefox HTTP/2 configuration.
fn firefox_h2_options() -> Http2Options {
    Http2Options::builder()
        .initial_window_size(131072)
        .max_header_list_size(65536)
        .header_table_size(65536)
        .enable_push(true)
        .build()
}

/// Firefox default headers.
fn firefox_headers(version: &str, is_private: bool, is_android: bool) -> HeaderMap {
    let mut headers = HeaderMap::new();

    let ua = if is_android {
        format!(
            "Mozilla/5.0 (Android 14; Mobile; rv:{}) Gecko/{} Firefox/{}",
            version, version, version
        )
    } else {
        format!(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:{}) Gecko/20100101 Firefox/{}",
            version, version
        )
    };

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
        HeaderValue::from_static("gzip, deflate, br, zstd"),
    );
    headers.insert(header::CONNECTION, HeaderValue::from_static("keep-alive"));
    headers.insert(
        header::UPGRADE_INSECURE_REQUESTS,
        HeaderValue::from_static("1"),
    );

    // Sec-Fetch headers
    headers.insert("sec-fetch-dest", HeaderValue::from_static("document"));
    headers.insert("sec-fetch-mode", HeaderValue::from_static("navigate"));
    headers.insert("sec-fetch-site", HeaderValue::from_static("none"));
    headers.insert("sec-fetch-user", HeaderValue::from_static("?1"));
    headers.insert("priority", HeaderValue::from_static("u=1"));

    // Private mode hint (not actually sent, but useful for testing)
    if is_private {
        headers.insert("dnt", HeaderValue::from_static("1"));
    }

    headers
}
