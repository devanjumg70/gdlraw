//! Browser Profile Definitions.
//!
//! Predefined browser profiles with correct User-Agent strings, headers,
//! and TLS/HTTP settings for different platforms.

use http::header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE};
use std::fmt;

// =============================================================================
// Profile Type Enum
// =============================================================================

/// Browser profile category for client selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProfileType {
    /// Desktop Chrome on Windows (default).
    #[default]
    ChromeWindows,
    /// Desktop Chrome on macOS.
    ChromeMac,
    /// Desktop Chrome on Linux.
    ChromeLinux,
    /// Desktop Firefox on Windows.
    FirefoxWindows,
    /// Desktop Safari on macOS.
    SafariMac,
    /// Desktop Edge on Windows.
    EdgeWindows,
    /// Mobile Chrome on Android.
    ChromeAndroid,
    /// Mobile Safari on iOS.
    SafariIos,
}

impl ProfileType {
    /// Check if this is a mobile profile.
    pub fn is_mobile(&self) -> bool {
        matches!(
            self,
            ProfileType::ChromeAndroid | ProfileType::SafariIos
        )
    }

    /// Check if this is a desktop profile.
    pub fn is_desktop(&self) -> bool {
        matches!(
            self,
            ProfileType::ChromeWindows
                | ProfileType::ChromeMac
                | ProfileType::ChromeLinux
                | ProfileType::FirefoxWindows
                | ProfileType::SafariMac
                | ProfileType::EdgeWindows
        )
    }
}

impl fmt::Display for ProfileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            ProfileType::ChromeWindows => "Chrome/Windows",
            ProfileType::ChromeMac => "Chrome/macOS",
            ProfileType::ChromeLinux => "Chrome/Linux",
            ProfileType::FirefoxWindows => "Firefox/Windows",
            ProfileType::SafariMac => "Safari/macOS",
            ProfileType::EdgeWindows => "Edge/Windows",
            ProfileType::ChromeAndroid => "Chrome/Android",
            ProfileType::SafariIos => "Safari/iOS",
        };
        write!(f, "{name}")
    }
}

// =============================================================================
// Browser Profile
// =============================================================================

/// Complete browser profile with User-Agent and default headers.
#[derive(Debug, Clone)]
pub struct BrowserProfile {
    /// Profile type identifier.
    pub profile_type: ProfileType,
    /// User-Agent string.
    pub user_agent: &'static str,
    /// Accept header value.
    pub accept: &'static str,
    /// Accept-Language header value.
    pub accept_language: &'static str,
    /// Accept-Encoding header value.
    pub accept_encoding: &'static str,
    /// Sec-Fetch-Dest header.
    pub sec_fetch_dest: Option<&'static str>,
    /// Sec-Fetch-Mode header.
    pub sec_fetch_mode: Option<&'static str>,
    /// Sec-Fetch-Site header.
    pub sec_fetch_site: Option<&'static str>,
    /// Sec-CH-UA header for Chrome.
    pub sec_ch_ua: Option<&'static str>,
    /// Sec-CH-UA-Mobile header.
    pub sec_ch_ua_mobile: Option<&'static str>,
    /// Sec-CH-UA-Platform header.
    pub sec_ch_ua_platform: Option<&'static str>,
}

impl BrowserProfile {
    // =========================================================================
    // Desktop Profiles
    // =========================================================================

    /// Chrome 124 on Windows.
    pub const fn chrome_windows() -> Self {
        Self {
            profile_type: ProfileType::ChromeWindows,
            user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
            accept: "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8",
            accept_language: "en-US,en;q=0.9",
            accept_encoding: "gzip, deflate, br, zstd",
            sec_fetch_dest: Some("document"),
            sec_fetch_mode: Some("navigate"),
            sec_fetch_site: Some("none"),
            sec_ch_ua: Some("\"Chromium\";v=\"124\", \"Google Chrome\";v=\"124\", \"Not-A.Brand\";v=\"99\""),
            sec_ch_ua_mobile: Some("?0"),
            sec_ch_ua_platform: Some("\"Windows\""),
        }
    }

    /// Chrome 124 on macOS.
    pub const fn chrome_mac() -> Self {
        Self {
            profile_type: ProfileType::ChromeMac,
            user_agent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
            accept: "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8",
            accept_language: "en-US,en;q=0.9",
            accept_encoding: "gzip, deflate, br, zstd",
            sec_fetch_dest: Some("document"),
            sec_fetch_mode: Some("navigate"),
            sec_fetch_site: Some("none"),
            sec_ch_ua: Some("\"Chromium\";v=\"124\", \"Google Chrome\";v=\"124\", \"Not-A.Brand\";v=\"99\""),
            sec_ch_ua_mobile: Some("?0"),
            sec_ch_ua_platform: Some("\"macOS\""),
        }
    }

    /// Chrome 124 on Linux.
    pub const fn chrome_linux() -> Self {
        Self {
            profile_type: ProfileType::ChromeLinux,
            user_agent: "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
            accept: "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8",
            accept_language: "en-US,en;q=0.9",
            accept_encoding: "gzip, deflate, br, zstd",
            sec_fetch_dest: Some("document"),
            sec_fetch_mode: Some("navigate"),
            sec_fetch_site: Some("none"),
            sec_ch_ua: Some("\"Chromium\";v=\"124\", \"Google Chrome\";v=\"124\", \"Not-A.Brand\";v=\"99\""),
            sec_ch_ua_mobile: Some("?0"),
            sec_ch_ua_platform: Some("\"Linux\""),
        }
    }

    /// Firefox 128 on Windows.
    pub const fn firefox_windows() -> Self {
        Self {
            profile_type: ProfileType::FirefoxWindows,
            user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:128.0) Gecko/20100101 Firefox/128.0",
            accept: "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/png,image/svg+xml,*/*;q=0.8",
            accept_language: "en-US,en;q=0.5",
            accept_encoding: "gzip, deflate, br, zstd",
            sec_fetch_dest: Some("document"),
            sec_fetch_mode: Some("navigate"),
            sec_fetch_site: Some("none"),
            sec_ch_ua: None,
            sec_ch_ua_mobile: None,
            sec_ch_ua_platform: None,
        }
    }

    /// Safari 17 on macOS.
    pub const fn safari_mac() -> Self {
        Self {
            profile_type: ProfileType::SafariMac,
            user_agent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.4 Safari/605.1.15",
            accept: "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            accept_language: "en-US,en;q=0.9",
            accept_encoding: "gzip, deflate, br",
            sec_fetch_dest: Some("document"),
            sec_fetch_mode: Some("navigate"),
            sec_fetch_site: Some("none"),
            sec_ch_ua: None,
            sec_ch_ua_mobile: None,
            sec_ch_ua_platform: None,
        }
    }
    
    /// Edge 124 on Windows.
    pub const fn edge_windows() -> Self {
        Self {
            profile_type: ProfileType::EdgeWindows,
            user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36 Edg/124.0.0.0",
            accept: "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8",
            accept_language: "en-US,en;q=0.9",
            accept_encoding: "gzip, deflate, br, zstd",
            sec_fetch_dest: Some("document"),
            sec_fetch_mode: Some("navigate"),
            sec_fetch_site: Some("none"),
            sec_ch_ua: Some("\"Chromium\";v=\"124\", \"Microsoft Edge\";v=\"124\", \"Not-A.Brand\";v=\"99\""),
            sec_ch_ua_mobile: Some("?0"),
            sec_ch_ua_platform: Some("\"Windows\""),
        }
    }

    // =========================================================================
    // Mobile Profiles
    // =========================================================================

    /// Chrome on Android.
    pub const fn chrome_android() -> Self {
        Self {
            profile_type: ProfileType::ChromeAndroid,
            user_agent: "Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Mobile Safari/537.36",
            accept: "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8",
            accept_language: "en-US,en;q=0.9",
            accept_encoding: "gzip, deflate, br, zstd",
            sec_fetch_dest: Some("document"),
            sec_fetch_mode: Some("navigate"),
            sec_fetch_site: Some("none"),
            sec_ch_ua: Some("\"Chromium\";v=\"124\", \"Google Chrome\";v=\"124\", \"Not-A.Brand\";v=\"99\""),
            sec_ch_ua_mobile: Some("?1"),
            sec_ch_ua_platform: Some("\"Android\""),
        }
    }

    /// Safari on iOS.
    pub const fn safari_ios() -> Self {
        Self {
            profile_type: ProfileType::SafariIos,
            user_agent: "Mozilla/5.0 (iPhone; CPU iPhone OS 17_4 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.4 Mobile/15E148 Safari/604.1",
            accept: "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            accept_language: "en-US,en;q=0.9",
            accept_encoding: "gzip, deflate, br",
            sec_fetch_dest: Some("document"),
            sec_fetch_mode: Some("navigate"),
            sec_fetch_site: Some("none"),
            sec_ch_ua: None,
            sec_ch_ua_mobile: None,
            sec_ch_ua_platform: None,
        }
    }
    
    // =========================================================================
    // Methods
    // =========================================================================

    /// Get profile by type.
    pub fn from_type(profile_type: ProfileType) -> Self {
        match profile_type {
            ProfileType::ChromeWindows => Self::chrome_windows(),
            ProfileType::ChromeMac => Self::chrome_mac(),
            ProfileType::ChromeLinux => Self::chrome_linux(),
            ProfileType::FirefoxWindows => Self::firefox_windows(),
            ProfileType::SafariMac => Self::safari_mac(),
            ProfileType::EdgeWindows => Self::edge_windows(),
            ProfileType::ChromeAndroid => Self::chrome_android(),
            ProfileType::SafariIos => Self::safari_ios(),
        }
    }
    
    /// Build default headers for this profile.
    pub fn default_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::with_capacity(12);

        if let Ok(val) = HeaderValue::from_str(self.accept) {
            headers.insert(ACCEPT, val);
        }
        if let Ok(val) = HeaderValue::from_str(self.accept_language) {
            headers.insert(ACCEPT_LANGUAGE, val);
        }
        if let Ok(val) = HeaderValue::from_str(self.accept_encoding) {
            headers.insert(ACCEPT_ENCODING, val);
        }

        if let Some(dest) = self.sec_fetch_dest {
            if let Ok(val) = HeaderValue::from_str(dest) {
                headers.insert("Sec-Fetch-Dest", val);
            }
        }
        if let Some(mode) = self.sec_fetch_mode {
            if let Ok(val) = HeaderValue::from_str(mode) {
                headers.insert("Sec-Fetch-Mode", val);
            }
        }
        if let Some(site) = self.sec_fetch_site {
            if let Ok(val) = HeaderValue::from_str(site) {
                headers.insert("Sec-Fetch-Site", val);
            }
        }

        if let Some(ua) = self.sec_ch_ua {
            if let Ok(val) = HeaderValue::from_str(ua) {
                headers.insert("Sec-CH-UA", val);
            }
        }
        if let Some(mobile) = self.sec_ch_ua_mobile {
            if let Ok(val) = HeaderValue::from_str(mobile) {
                headers.insert("Sec-CH-UA-Mobile", val);
            }
        }
        if let Some(platform) = self.sec_ch_ua_platform {
            if let Ok(val) = HeaderValue::from_str(platform) {
                headers.insert("Sec-CH-UA-Platform", val);
            }
        }

        headers
    }
}
