//! HTTP/2 SETTINGS configuration for browser fingerprinting.
//!
//! This module provides configurable HTTP/2 SETTINGS values that match
//! different browser profiles. Anti-bot systems fingerprint these values,
//! so matching a real browser is critical for stealth.
//!
//! Based on Chromium's spdy_session.h defaults and real browser captures.

/// HTTP/2 SETTINGS configuration.
/// Values are sent in the SETTINGS frame during connection setup.
#[derive(Debug, Clone, Copy)]
pub struct H2Settings {
    /// SETTINGS_HEADER_TABLE_SIZE (0x1) - HPACK dynamic table size
    pub header_table_size: u32,
    /// SETTINGS_ENABLE_PUSH (0x2) - Server push enabled
    pub enable_push: bool,
    /// SETTINGS_MAX_CONCURRENT_STREAMS (0x3)
    pub max_concurrent_streams: u32,
    /// SETTINGS_INITIAL_WINDOW_SIZE (0x4) - Flow control window
    pub initial_window_size: u32,
    /// SETTINGS_MAX_FRAME_SIZE (0x5) - Maximum frame payload
    pub max_frame_size: u32,
    /// SETTINGS_MAX_HEADER_LIST_SIZE (0x6) - Maximum header block size
    pub max_header_list_size: u32,
}

impl Default for H2Settings {
    fn default() -> Self {
        Self::chrome()
    }
}

impl H2Settings {
    /// Chrome 120+ HTTP/2 SETTINGS.
    /// Based on Chromium source (spdy_session.h) and live captures.
    pub fn chrome() -> Self {
        Self {
            header_table_size: 65536,
            enable_push: false, // Chrome disabled push in 2022
            max_concurrent_streams: 1000,
            initial_window_size: 6291456, // 6MB - Chrome's aggressive window
            max_frame_size: 16384,        // 16KB - RFC default
            max_header_list_size: 262144, // 256KB
        }
    }

    /// Firefox 120+ HTTP/2 SETTINGS.
    pub fn firefox() -> Self {
        Self {
            header_table_size: 65536,
            enable_push: false,
            max_concurrent_streams: 100,
            initial_window_size: 65535, // RFC default 64KB
            max_frame_size: 16384,
            max_header_list_size: 65536,
        }
    }

    /// Safari 17+ HTTP/2 SETTINGS.
    pub fn safari() -> Self {
        Self {
            header_table_size: 4096, // Safari uses smaller table
            enable_push: false,
            max_concurrent_streams: 100,
            initial_window_size: 65535,
            max_frame_size: 16384,
            max_header_list_size: 0, // Safari doesn't send this
        }
    }

    /// Custom settings - use with caution.
    pub fn custom(
        header_table_size: u32,
        enable_push: bool,
        max_concurrent_streams: u32,
        initial_window_size: u32,
        max_frame_size: u32,
        max_header_list_size: u32,
    ) -> Self {
        Self {
            header_table_size,
            enable_push,
            max_concurrent_streams,
            initial_window_size,
            max_frame_size,
            max_header_list_size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chrome_defaults() {
        let settings = H2Settings::chrome();
        assert_eq!(settings.header_table_size, 65536);
        assert!(!settings.enable_push);
        assert_eq!(settings.max_concurrent_streams, 1000);
        assert_eq!(settings.initial_window_size, 6291456);
        assert_eq!(settings.max_frame_size, 16384);
        assert_eq!(settings.max_header_list_size, 262144);
    }

    #[test]
    fn test_firefox_defaults() {
        let settings = H2Settings::firefox();
        assert_eq!(settings.header_table_size, 65536);
        assert_eq!(settings.max_concurrent_streams, 100);
        assert_eq!(settings.initial_window_size, 65535);
    }

    #[test]
    fn test_safari_defaults() {
        let settings = H2Settings::safari();
        assert_eq!(settings.header_table_size, 4096);
        assert_eq!(settings.max_header_list_size, 0);
    }

    #[test]
    fn test_default_is_chrome() {
        let default = H2Settings::default();
        let chrome = H2Settings::chrome();
        assert_eq!(default.initial_window_size, chrome.initial_window_size);
    }

    #[test]
    fn test_custom_settings() {
        let s = H2Settings::custom(4096, true, 50, 32768, 8192, 16384);
        assert_eq!(s.header_table_size, 4096);
        assert!(s.enable_push);
        assert_eq!(s.max_concurrent_streams, 50);
    }

    #[test]
    fn test_settings_are_copy() {
        let s1 = H2Settings::chrome();
        let s2 = s1; // Copy
        assert_eq!(s1.initial_window_size, s2.initial_window_size);
    }

    #[test]
    fn test_chrome_larger_window_than_firefox() {
        assert!(
            H2Settings::chrome().initial_window_size > H2Settings::firefox().initial_window_size
        );
    }
}
