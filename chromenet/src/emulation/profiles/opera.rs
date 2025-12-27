//! Opera browser profiles.
//!
//! Provides emulation configurations for Opera browser.
//! Opera is Chromium-based with similar TLS/H2 fingerprints.

use crate::emulation::{Emulation, EmulationFactory, Http2Options};
use crate::socket::tls::{AlpnProtocol, TlsOptions, TlsVersion};
use http::{header, HeaderMap, HeaderValue};

/// Opera browser versions for emulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Opera {
    /// Opera 116 (Chromium 131)
    V116,
    /// Opera 117 (Chromium 132)
    V117,
    /// Opera 118 (Chromium 133)
    V118,
    /// Opera 119 (Chromium 134)
    V119,
}

impl Default for Opera {
    fn default() -> Self {
        Opera::V119
    }
}

impl EmulationFactory for Opera {
    fn emulation(self) -> Emulation {
        match self {
            Opera::V116 => opera_v116(),
            Opera::V117 => opera_v117(),
            Opera::V118 => opera_v118(),
            Opera::V119 => opera_v119(),
        }
    }
}

/// Create Opera 116 emulation (Chromium 131).
pub fn opera_v116() -> Emulation {
    opera_emulation(
        "116.0.0.0",
        "131",
        r#""Opera";v="116", "Chromium";v="131", "Not_A Brand";v="24""#,
    )
}

/// Create Opera 117 emulation (Chromium 132).
pub fn opera_v117() -> Emulation {
    opera_emulation(
        "117.0.0.0",
        "132",
        r#""Not A(Brand";v="8", "Chromium";v="132", "Opera";v="117""#,
    )
}

/// Create Opera 118 emulation (Chromium 133).
pub fn opera_v118() -> Emulation {
    opera_emulation(
        "118.0.0.0",
        "133",
        r#""Not(A:Brand";v="99", "Opera";v="118", "Chromium";v="133""#,
    )
}

/// Create Opera 119 emulation (Chromium 134).
pub fn opera_v119() -> Emulation {
    opera_emulation(
        "119.0.0.0",
        "134",
        r#""Chromium";v="134", "Not:A-Brand";v="24", "Opera";v="119""#,
    )
}

/// Create Opera emulation (Chromium-based).
fn opera_emulation(opera_version: &str, chromium_version: &str, sec_ch_ua: &str) -> Emulation {
    // Opera uses Chromium's TLS stack
    let tls = TlsOptions::builder()
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
        .build();

    // Same H2 settings as Chrome
    let h2 = Http2Options::builder()
        .initial_window_size(6291456)
        .max_header_list_size(262144)
        .header_table_size(65536)
        .enable_push(false)
        .build();

    let mut headers = HeaderMap::new();

    // User-Agent with Opera branding
    let ua = format!(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{}.0.0.0 Safari/537.36 OPR/{}",
        chromium_version, opera_version
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

    // Sec-CH-UA headers with Opera branding
    if let Ok(val) = HeaderValue::from_str(sec_ch_ua) {
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

    Emulation::builder()
        .tls_options(tls)
        .http2_options(h2)
        .headers(headers)
        .build()
}
