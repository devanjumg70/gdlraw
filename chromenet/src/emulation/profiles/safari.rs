//! Safari browser profiles.
//!
//! Provides emulation configurations for various Safari versions.

use crate::emulation::{Emulation, EmulationFactory, Http2Options};
use crate::socket::tls::{AlpnProtocol, TlsOptions, TlsVersion};
use http::{header, HeaderMap, HeaderValue};

/// Safari browser versions for emulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Safari {
    /// Safari 15.3
    V15_3,
    /// Safari 15.6
    V15_6,
    /// Safari 16
    V16,
    /// Safari 16.5
    V16_5,
    /// Safari 17
    V17,
    /// Safari 17.2
    V17_2,
    /// Safari 17.5
    V17_5,
    /// Safari 17.6
    V17_6,
    /// Safari 18
    V18,
    /// Safari 18.2
    V18_2,
    /// Safari 18.3
    V18_3,
    /// Safari 18.5 (latest)
    V18_5,
    /// Safari on iOS 17
    IOS17,
    /// Safari on iOS 18
    IOS18,
    /// Safari on iPad 18
    IPad18,
}

impl Default for Safari {
    fn default() -> Self {
        Safari::V18_5
    }
}

impl EmulationFactory for Safari {
    fn emulation(self) -> Emulation {
        safari_emulation(self.version_string(), self.platform())
    }
}

/// Safari platform type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafariPlatform {
    /// macOS desktop
    MacOS,
    /// iPhone/iOS
    IOS,
    /// iPad
    IPad,
}

impl Safari {
    /// Get version string.
    pub fn version_string(self) -> &'static str {
        match self {
            Safari::V15_3 => "15.3",
            Safari::V15_6 => "15.6.1",
            Safari::V16 => "16.0",
            Safari::V16_5 => "16.5",
            Safari::V17 => "17.0",
            Safari::V17_2 => "17.2.1",
            Safari::V17_5 => "17.5",
            Safari::V17_6 => "17.6",
            Safari::V18 => "18.0",
            Safari::V18_2 => "18.2",
            Safari::V18_3 => "18.3",
            Safari::V18_5 => "18.5",
            Safari::IOS17 => "17.0",
            Safari::IOS18 => "18.0",
            Safari::IPad18 => "18.0",
        }
    }

    /// Get platform.
    pub fn platform(self) -> SafariPlatform {
        match self {
            Safari::IOS17 | Safari::IOS18 => SafariPlatform::IOS,
            Safari::IPad18 => SafariPlatform::IPad,
            _ => SafariPlatform::MacOS,
        }
    }

    /// Get all supported versions.
    pub fn all_versions() -> &'static [Safari] {
        &[
            Safari::V15_3,
            Safari::V15_6,
            Safari::V16,
            Safari::V16_5,
            Safari::V17,
            Safari::V17_2,
            Safari::V17_5,
            Safari::V17_6,
            Safari::V18,
            Safari::V18_2,
            Safari::V18_3,
            Safari::V18_5,
            Safari::IOS17,
            Safari::IOS18,
            Safari::IPad18,
        ]
    }
}

/// Create Safari emulation for a specific version.
fn safari_emulation(version: &'static str, platform: SafariPlatform) -> Emulation {
    let tls = safari_tls_options();
    let h2 = safari_h2_options();
    let headers = safari_headers(version, platform);

    Emulation::builder()
        .tls_options(tls)
        .http2_options(h2)
        .headers(headers)
        .build()
}

/// Safari TLS configuration (SecureTransport).
fn safari_tls_options() -> TlsOptions {
    TlsOptions::builder()
        .alpn_protocols([AlpnProtocol::HTTP2, AlpnProtocol::HTTP1])
        .min_tls_version(TlsVersion::TLS_1_2)
        .max_tls_version(TlsVersion::TLS_1_3)
        .cipher_list(
            "TLS_AES_128_GCM_SHA256:TLS_AES_256_GCM_SHA384:TLS_CHACHA20_POLY1305_SHA256:\
             ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-ECDSA-AES128-GCM-SHA256:\
             ECDHE-ECDSA-CHACHA20-POLY1305:\
             ECDHE-RSA-AES256-GCM-SHA384:ECDHE-RSA-AES128-GCM-SHA256:\
             ECDHE-RSA-CHACHA20-POLY1305",
        )
        // Safari prefers P-256 over X25519
        .curves_list("P-256:P-384:P-521:X25519")
        .sigalgs_list(
            "ecdsa_secp256r1_sha256:rsa_pss_rsae_sha256:\
             ecdsa_secp384r1_sha384:rsa_pss_rsae_sha384:\
             ecdsa_secp521r1_sha512:rsa_pss_rsae_sha512:\
             rsa_pkcs1_sha256:rsa_pkcs1_sha384:rsa_pkcs1_sha512",
        )
        .grease_enabled(false)
        .permute_extensions(false)
        .enable_ocsp_stapling(true)
        .session_ticket(true)
        .build()
}

/// Safari HTTP/2 configuration.
fn safari_h2_options() -> Http2Options {
    Http2Options::builder()
        .initial_window_size(4194304) // 4MB
        .header_table_size(4096)
        .enable_push(true)
        .build()
}

/// Safari default headers.
fn safari_headers(version: &str, platform: SafariPlatform) -> HeaderMap {
    let mut headers = HeaderMap::new();

    let ua = match platform {
        SafariPlatform::IOS => {
            let ios_ver = version.split('.').next().unwrap_or("18");
            format!(
                "Mozilla/5.0 (iPhone; CPU iPhone OS {}_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/{} Mobile/15E148 Safari/604.1",
                ios_ver, version
            )
        }
        SafariPlatform::IPad => {
            let ios_ver = version.split('.').next().unwrap_or("18");
            format!(
                "Mozilla/5.0 (iPad; CPU OS {}_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/{} Mobile/15E148 Safari/604.1",
                ios_ver, version
            )
        }
        SafariPlatform::MacOS => {
            format!(
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/{} Safari/605.1.15",
                version
            )
        }
    };

    if let Ok(val) = HeaderValue::from_str(&ua) {
        headers.insert(header::USER_AGENT, val);
    }
    headers.insert(
        header::ACCEPT,
        HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"),
    );
    headers.insert(
        header::ACCEPT_LANGUAGE,
        HeaderValue::from_static("en-US,en;q=0.9"),
    );
    headers.insert(
        header::ACCEPT_ENCODING,
        HeaderValue::from_static("gzip, deflate, br"),
    );

    // Safari sec-fetch headers (minimal)
    headers.insert("sec-fetch-dest", HeaderValue::from_static("document"));
    headers.insert("sec-fetch-mode", HeaderValue::from_static("navigate"));
    headers.insert("sec-fetch-site", HeaderValue::from_static("none"));

    headers
}
