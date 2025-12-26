//! Chromium cookie database constants and utilities.
//!
//! This module provides constants and functions that exactly match
//! Chromium's cookie storage implementation in `//net/extras/sqlite`.
//!
//! ## Reference Files
//! - `net/extras/sqlite/sqlite_persistent_cookie_store.cc`
//! - `components/os_crypt/sync/os_crypt_linux.cc`
//! - `net/cookies/canonical_cookie.cc`
//!
//! ## Database Version
//! Current Chromium cookie database version is 24 (as of Chromium 120+).

use time::OffsetDateTime;

/// Chromium uses microseconds since 1601-01-01 00:00:00 UTC (Windows FILETIME epoch).
/// This is the offset from Unix epoch (1970-01-01) in microseconds.
///
/// Reference: `base/time/time.h`
pub const CHROME_EPOCH_OFFSET_MICROS: i64 = 11_644_473_600_000_000;

/// Convert Chrome epoch (microseconds since 1601) to Unix timestamp.
///
/// Reference: `base/time/time_win.cc`
pub fn chrome_to_unix_timestamp(chrome_time: i64) -> Option<OffsetDateTime> {
    if chrome_time == 0 {
        return None;
    }

    let unix_micros = chrome_time - CHROME_EPOCH_OFFSET_MICROS;
    let unix_secs = unix_micros / 1_000_000;
    OffsetDateTime::from_unix_timestamp(unix_secs).ok()
}

/// Convert Unix timestamp to Chrome epoch (microseconds since 1601).
pub fn unix_to_chrome_timestamp(time: OffsetDateTime) -> i64 {
    let unix_secs = time.unix_timestamp();
    (unix_secs * 1_000_000) + CHROME_EPOCH_OFFSET_MICROS
}

/// Cookie priority levels matching Chromium's `CookiePriority` enum.
///
/// Reference: `net/cookies/cookie_constants.h`
pub mod priority {
    pub const LOW: i32 = 0;
    pub const MEDIUM: i32 = 1;
    pub const HIGH: i32 = 2;
}

/// Cookie SameSite values matching Chromium's `CookieSameSite` enum.
///
/// Reference: `net/cookies/cookie_constants.h`
pub mod samesite {
    pub const UNSPECIFIED: i32 = -1;
    pub const NO_RESTRICTION: i32 = 0;
    pub const LAX: i32 = 1;
    pub const STRICT: i32 = 2;
}

/// Source scheme values for cookies.
///
/// Reference: `net/cookies/cookie_constants.h`
pub mod source_scheme {
    pub const UNSET: i32 = 0;
    pub const NON_SECURE: i32 = 1;
    pub const SECURE: i32 = 2;
}

/// Cookie source type values.
///
/// Reference: `net/cookies/cookie_source_type.h`
pub mod source_type {
    pub const UNKNOWN: i32 = 0;
    pub const HTTP: i32 = 1;
    pub const SCRIPT: i32 = 2;
    pub const OTHER: i32 = 3;
}

/// Chromium cookie database schema version.
///
/// Reference: `net/extras/sqlite/sqlite_persistent_cookie_store.cc`
pub const COOKIE_DATABASE_VERSION: i32 = 24;

/// Column names in the Chromium cookies table.
///
/// Reference: `net/extras/sqlite/sqlite_persistent_cookie_store.cc`
pub const COOKIE_COLUMNS: &[&str] = &[
    "creation_utc",
    "host_key",
    "top_frame_site_key",
    "name",
    "value",
    "encrypted_value",
    "path",
    "expires_utc",
    "is_secure",
    "is_httponly",
    "last_access_utc",
    "has_expires",
    "is_persistent",
    "priority",
    "samesite",
    "source_scheme",
    "source_port",
    "last_update_utc",
    "source_type",
    "has_cross_site_ancestor",
];

/// Encryption version prefixes.
///
/// Reference: `components/os_crypt/sync/os_crypt_linux.cc`
pub mod encryption {
    /// Linux v10: Hardcoded key from "peanuts"
    pub const V10_PREFIX: &[u8] = b"v10";

    /// Linux v11: Key from GNOME Keyring
    pub const V11_PREFIX: &[u8] = b"v11";

    /// Default password for v10 encryption
    pub const V10_PASSWORD: &[u8] = b"peanuts";

    /// Salt used for all PBKDF2 key derivation
    pub const CHROME_SALT: &[u8] = b"saltysalt";

    /// PBKDF2 iterations for Linux v10/v11
    pub const LINUX_ITERATIONS: u32 = 1;

    /// PBKDF2 iterations for macOS
    pub const MACOS_ITERATIONS: u32 = 1003;

    /// AES-CBC IV (16 space characters)
    pub const AES_CBC_IV: [u8; 16] = [0x20; 16];
}

/// Browser user data directory paths.
///
/// These match the standard locations used by each browser.
pub mod paths {
    /// Linux Chrome user data path template
    pub const LINUX_CHROME: &str = ".config/google-chrome";

    /// Linux Chromium user data path template
    pub const LINUX_CHROMIUM: &str = ".config/chromium";

    /// Linux Firefox profiles path template
    pub const LINUX_FIREFOX: &str = ".mozilla/firefox";

    /// macOS Chrome user data path template
    pub const MACOS_CHROME: &str = "Library/Application Support/Google/Chrome";

    /// macOS Safari cookies path
    pub const MACOS_SAFARI: &str = "Library/Cookies/Cookies.binarycookies";

    /// Windows Chrome user data path (relative to %LOCALAPPDATA%)
    pub const WINDOWS_CHROME: &str = "Google/Chrome/User Data";

    /// Windows Firefox profiles path (relative to %APPDATA%)
    pub const WINDOWS_FIREFOX: &str = "Mozilla/Firefox/Profiles";
}

/// Keyring/Keychain service names for different browsers.
///
/// Reference: `components/os_crypt/sync/key_storage_libsecret.cc`
pub mod keyring {
    /// GNOME Keyring schema name for Chromium
    pub const CHROME_SCHEMA: &str = "chrome_libsecret_os_crypt_password_v2";

    /// macOS Keychain service for Chrome
    pub const MACOS_CHROME_SERVICE: &str = "Chrome Safe Storage";

    /// macOS Keychain service for Chromium
    pub const MACOS_CHROMIUM_SERVICE: &str = "Chromium Safe Storage";

    /// macOS Keychain service for Edge
    pub const MACOS_EDGE_SERVICE: &str = "Microsoft Edge Safe Storage";

    /// macOS Keychain service for Brave
    pub const MACOS_BRAVE_SERVICE: &str = "Brave Safe Storage";

    /// macOS Keychain service for Opera
    pub const MACOS_OPERA_SERVICE: &str = "Opera Safe Storage";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chrome_epoch_conversion() {
        // Test known timestamp: 2024-01-01 00:00:00 UTC
        // Unix timestamp: 1704067200
        // Chrome timestamp: 1704067200 * 1_000_000 + CHROME_EPOCH_OFFSET_MICROS
        let chrome_time = 1704067200_i64 * 1_000_000 + CHROME_EPOCH_OFFSET_MICROS;
        let result = chrome_to_unix_timestamp(chrome_time);
        assert!(result.is_some());
        let dt = result.unwrap();
        assert_eq!(dt.year(), 2024);
    }

    #[test]
    fn test_chrome_epoch_zero() {
        let result = chrome_to_unix_timestamp(0);
        assert!(result.is_none());
    }

    #[test]
    fn test_roundtrip_conversion() {
        let now = OffsetDateTime::now_utc();
        let chrome_time = unix_to_chrome_timestamp(now);
        let back = chrome_to_unix_timestamp(chrome_time);
        assert!(back.is_some());
        // Allow 1 second difference due to microsecond truncation
        let diff = (now.unix_timestamp() - back.unwrap().unix_timestamp()).abs();
        assert!(diff <= 1);
    }

    #[test]
    fn test_encryption_constants() {
        assert_eq!(encryption::V10_PREFIX, b"v10");
        assert_eq!(encryption::V11_PREFIX, b"v11");
        assert_eq!(encryption::V10_PASSWORD, b"peanuts");
        assert_eq!(encryption::LINUX_ITERATIONS, 1);
        assert_eq!(encryption::MACOS_ITERATIONS, 1003);
    }

    #[test]
    fn test_samesite_constants() {
        assert_eq!(samesite::UNSPECIFIED, -1);
        assert_eq!(samesite::NO_RESTRICTION, 0);
        assert_eq!(samesite::LAX, 1);
        assert_eq!(samesite::STRICT, 2);
    }
}
