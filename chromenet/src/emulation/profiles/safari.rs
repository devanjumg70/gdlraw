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
    /// Safari 17
    V17,
    /// Safari 17.5
    V17_5,
    /// Safari 18
    V18,
    /// Safari 18.2
    V18_2,
    /// Safari on iOS 17
    IOS17,
    /// Safari on iOS 18
    IOS18,
}

impl Default for Safari {
    fn default() -> Self {
        Safari::V18_2
    }
}

impl EmulationFactory for Safari {
    fn emulation(self) -> Emulation {
        match self {
            Safari::V17 => safari_v17(),
            Safari::V17_5 => safari_v17_5(),
            Safari::V18 => safari_v18(),
            Safari::V18_2 => safari_v18_2(),
            Safari::IOS17 => safari_ios17(),
            Safari::IOS18 => safari_ios18(),
        }
    }
}

/// Create Safari v17 emulation.
pub fn safari_v17() -> Emulation {
    safari_emulation("17.0", false)
}

/// Create Safari v17.5 emulation.
pub fn safari_v17_5() -> Emulation {
    safari_emulation("17.5", false)
}

/// Create Safari v18 emulation.
pub fn safari_v18() -> Emulation {
    safari_emulation("18.0", false)
}

/// Create Safari v18.2 emulation.
pub fn safari_v18_2() -> Emulation {
    safari_emulation("18.2", false)
}

/// Create Safari iOS 17 emulation.
pub fn safari_ios17() -> Emulation {
    safari_emulation("17.0", true)
}

/// Create Safari iOS 18 emulation.
pub fn safari_ios18() -> Emulation {
    safari_emulation("18.0", true)
}

/// Create Safari emulation for a specific version.
fn safari_emulation(version: &str, is_ios: bool) -> Emulation {
    let tls = safari_tls_options();
    let h2 = safari_h2_options();
    let headers = safari_headers(version, is_ios);

    Emulation::builder()
        .tls_options(tls)
        .http2_options(h2)
        .headers(headers)
        .build()
}

/// Safari TLS configuration.
/// Safari uses Apple's SecureTransport, different from Chrome/Firefox.
fn safari_tls_options() -> TlsOptions {
    TlsOptions::builder()
        .alpn_protocols([AlpnProtocol::HTTP2, AlpnProtocol::HTTP1])
        .min_tls_version(TlsVersion::TLS_1_2)
        .max_tls_version(TlsVersion::TLS_1_3)
        // Safari cipher suite order
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
        .grease_enabled(false) // Safari doesn't use GREASE
        .permute_extensions(false)
        .enable_ocsp_stapling(true)
        .session_ticket(true)
        .build()
}

/// Safari HTTP/2 configuration.
fn safari_h2_options() -> Http2Options {
    Http2Options::builder()
        .initial_window_size(4194304) // Safari: 4MB
        .header_table_size(4096)
        .enable_push(true)
        .build()
}

/// Safari default headers.
fn safari_headers(version: &str, is_ios: bool) -> HeaderMap {
    let mut headers = HeaderMap::new();

    let ua = if is_ios {
        format!(
            "Mozilla/5.0 (iPhone; CPU iPhone OS {}_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/{} Mobile/15E148 Safari/604.1",
            version.split('.').next().unwrap_or("18"),
            version
        )
    } else {
        format!(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/{} Safari/605.1.15",
            version
        )
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

    // Safari-specific (no sec-ch-ua headers, minimal sec-fetch)
    headers.insert("sec-fetch-dest", HeaderValue::from_static("document"));
    headers.insert("sec-fetch-mode", HeaderValue::from_static("navigate"));
    headers.insert("sec-fetch-site", HeaderValue::from_static("none"));

    headers
}
