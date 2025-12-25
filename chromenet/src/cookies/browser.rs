//! Browser cookie extraction from Chrome/Firefox SQLite databases.
//!
//! Reads cookies from local browser databases for session reuse.
//! Supports Chrome, Chromium, Edge, Brave, Opera, Firefox, and Safari.
//!
//! ## Encryption Support
//! - **Linux v10**: Fully supported (hardcoded key + empty key fallback)
//! - **Linux v11**: Requires keyring access (not yet implemented)
//! - **macOS**: Requires Keychain access (not yet implemented)
//! - **Windows**: Requires DPAPI (not yet implemented)

use crate::base::neterror::NetError;
use crate::cookies::canonical_cookie::{CanonicalCookie, CookiePriority, SameSite};
use crate::cookies::error::{CookieExtractionError, CookieResult};
use crate::cookies::oscrypt;
use std::path::PathBuf;
use time::OffsetDateTime;

/// Supported browsers for cookie extraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Browser {
    /// Google Chrome
    Chrome,
    /// Chromium (open-source Chrome)
    Chromium,
    /// Microsoft Edge (Chromium-based)
    Edge,
    /// Brave Browser
    Brave,
    /// Opera Browser
    Opera,
    /// Mozilla Firefox
    Firefox,
    /// Apple Safari (macOS only)
    Safari,
}

impl Browser {
    /// Returns true if this is a Chromium-based browser.
    pub fn is_chromium_based(&self) -> bool {
        matches!(
            self,
            Browser::Chrome | Browser::Chromium | Browser::Edge | Browser::Brave | Browser::Opera
        )
    }

    /// Returns all Chromium-based browsers.
    pub fn all_chromium() -> &'static [Browser] {
        &[
            Browser::Chrome,
            Browser::Chromium,
            Browser::Edge,
            Browser::Brave,
            Browser::Opera,
        ]
    }
}

/// Reader for browser cookie databases.
pub struct BrowserCookieReader {
    browser: Browser,
    profile: Option<String>,
    domain_filter: Option<String>,
}

impl BrowserCookieReader {
    /// Create a new reader for the specified browser.
    pub fn new(browser: Browser) -> Self {
        Self {
            browser,
            profile: None,
            domain_filter: None,
        }
    }

    /// Use a specific profile (default: "Default" for Chrome, first profile for Firefox).
    pub fn with_profile(mut self, profile: impl Into<String>) -> Self {
        self.profile = Some(profile.into());
        self
    }

    /// Filter cookies by domain.
    pub fn domain(mut self, domain: impl Into<String>) -> Self {
        self.domain_filter = Some(domain.into());
        self
    }

    /// Get the path to the browser's cookie database.
    pub fn get_db_path(&self) -> Option<PathBuf> {
        match self.browser {
            Browser::Chrome => self.chromium_cookie_path("google-chrome", "Google/Chrome"),
            Browser::Chromium => self.chromium_cookie_path("chromium", "Chromium"),
            Browser::Edge => self.chromium_cookie_path("microsoft-edge", "Microsoft/Edge"),
            Browser::Brave => self
                .chromium_cookie_path("BraveSoftware/Brave-Browser", "BraveSoftware/Brave-Browser"),
            Browser::Opera => self.chromium_cookie_path("opera", "com.operasoftware.Opera"),
            Browser::Firefox => self.firefox_cookie_path(),
            Browser::Safari => self.safari_cookie_path(),
        }
    }

    fn chromium_cookie_path(&self, linux_name: &str, _macos_name: &str) -> Option<PathBuf> {
        let profile = self.profile.as_deref().unwrap_or("Default");

        #[cfg(target_os = "linux")]
        {
            let home = std::env::var("HOME").ok()?;
            Some(PathBuf::from(format!(
                "{}/.config/{}/{}/Cookies",
                home, linux_name, profile
            )))
        }

        #[cfg(target_os = "macos")]
        {
            let home = std::env::var("HOME").ok()?;
            Some(PathBuf::from(format!(
                "{}/Library/Application Support/{}/{}/Cookies",
                home, macos_name, profile
            )))
        }

        #[cfg(target_os = "windows")]
        {
            let local_app_data = std::env::var("LOCALAPPDATA").ok()?;
            Some(PathBuf::from(format!(
                "{}/{}/User Data/{}/Network/Cookies",
                local_app_data, macos_name, profile
            )))
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            let _ = (linux_name, macos_name, profile);
            None
        }
    }

    fn safari_cookie_path(&self) -> Option<PathBuf> {
        #[cfg(target_os = "macos")]
        {
            let home = std::env::var("HOME").ok()?;
            Some(PathBuf::from(format!(
                "{}/Library/Cookies/Cookies.binarycookies",
                home
            )))
        }

        #[cfg(not(target_os = "macos"))]
        {
            None // Safari is macOS-only
        }
    }

    fn firefox_cookie_path(&self) -> Option<PathBuf> {
        #[cfg(target_os = "linux")]
        {
            let home = std::env::var("HOME").ok()?;
            let firefox_dir = PathBuf::from(format!("{}/.mozilla/firefox", home));

            // Find first .default profile if no profile specified
            if let Some(profile) = &self.profile {
                return Some(firefox_dir.join(profile).join("cookies.sqlite"));
            }

            // Auto-detect profile
            if let Ok(entries) = std::fs::read_dir(&firefox_dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.ends_with(".default") || name.ends_with(".default-release") {
                        return Some(entry.path().join("cookies.sqlite"));
                    }
                }
            }
            None
        }

        #[cfg(target_os = "macos")]
        {
            let home = std::env::var("HOME").ok()?;
            let firefox_dir = PathBuf::from(format!(
                "{}/Library/Application Support/Firefox/Profiles",
                home
            ));

            if let Some(profile) = &self.profile {
                return Some(firefox_dir.join(profile).join("cookies.sqlite"));
            }

            // Auto-detect
            if let Ok(entries) = std::fs::read_dir(&firefox_dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.ends_with(".default") || name.ends_with(".default-release") {
                        return Some(entry.path().join("cookies.sqlite"));
                    }
                }
            }
            None
        }

        #[cfg(target_os = "windows")]
        {
            let app_data = std::env::var("APPDATA").ok()?;
            let firefox_dir = PathBuf::from(format!("{}/Mozilla/Firefox/Profiles", app_data));

            if let Some(profile) = &self.profile {
                return Some(firefox_dir.join(profile).join("cookies.sqlite"));
            }

            if let Ok(entries) = std::fs::read_dir(&firefox_dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.ends_with(".default") || name.ends_with(".default-release") {
                        return Some(entry.path().join("cookies.sqlite"));
                    }
                }
            }
            None
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            None
        }
    }

    /// Read all cookies from the browser database.
    pub fn read_cookies(&self) -> Result<Vec<CanonicalCookie>, NetError> {
        let db_path = self.get_db_path().ok_or(NetError::FileNotFound)?;

        if !db_path.exists() {
            return Err(NetError::FileNotFound);
        }

        match self.browser {
            Browser::Chrome
            | Browser::Chromium
            | Browser::Edge
            | Browser::Brave
            | Browser::Opera => self.read_chromium_cookies(&db_path),
            Browser::Firefox => self.read_firefox_cookies(&db_path),
            Browser::Safari => Err(NetError::NotImplemented), // Binary format not yet supported
        }
    }

    /// Read cookies with better error handling.
    pub fn read_cookies_v2(&self) -> CookieResult<Vec<CanonicalCookie>> {
        let db_path = self
            .get_db_path()
            .ok_or_else(|| CookieExtractionError::BrowserNotFound(format!("{:?}", self.browser)))?;

        if !db_path.exists() {
            return Err(CookieExtractionError::DatabaseNotFound(db_path));
        }

        match self.browser {
            Browser::Chrome
            | Browser::Chromium
            | Browser::Edge
            | Browser::Brave
            | Browser::Opera => self.read_chromium_cookies_v2(&db_path),
            Browser::Firefox => self.read_firefox_cookies_v2(&db_path),
            Browser::Safari => Err(CookieExtractionError::PlatformNotSupported(
                "Safari binary cookies not yet implemented".into(),
            )),
        }
    }

    fn read_chromium_cookies(&self, path: &PathBuf) -> Result<Vec<CanonicalCookie>, NetError> {
        use rusqlite::{Connection, OpenFlags};

        let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|_| NetError::FileNotFound)?;

        let mut stmt = conn
            .prepare(
                "SELECT host_key, name, value, encrypted_value, path, expires_utc, is_secure, is_httponly, samesite
                 FROM cookies",
            )
            .map_err(|_| NetError::InvalidResponse)?;

        let mut cookies = Vec::new();
        let now = OffsetDateTime::now_utc();

        let mut rows = stmt.query([]).map_err(|_| NetError::InvalidResponse)?;

        while let Some(row) = rows.next().map_err(|_| NetError::InvalidResponse)? {
            let host_key: String = row.get(0).unwrap_or_default();
            let name: String = row.get(1).unwrap_or_default();
            let value: String = row.get(2).unwrap_or_default();
            let encrypted_value: Vec<u8> = row.get(3).unwrap_or_default();
            let path: String = row.get(4).unwrap_or_default();
            let expires_utc: i64 = row.get(5).unwrap_or(0);
            let is_secure: i32 = row.get(6).unwrap_or(0);
            let is_httponly: i32 = row.get(7).unwrap_or(0);
            let samesite: i32 = row.get(8).unwrap_or(-1);

            // Apply domain filter
            if let Some(ref filter) = self.domain_filter {
                if !host_key.ends_with(filter) && !host_key.trim_start_matches('.').eq(filter) {
                    continue;
                }
            }

            // Determine the cookie value
            let cookie_value = if !value.is_empty() {
                value
            } else if !encrypted_value.is_empty() {
                // Try to decrypt
                match oscrypt::decrypt_v10(&encrypted_value) {
                    Some(decrypted) => decrypted,
                    None => continue, // Skip if decryption fails
                }
            } else {
                continue; // Skip cookies with no value
            };

            let host_only = !host_key.starts_with('.');
            let cookie = CanonicalCookie {
                name,
                value: cookie_value,
                domain: host_key,
                path,
                expiration_time: chrome_time_to_offset(expires_utc),
                secure: is_secure != 0,
                http_only: is_httponly != 0,
                same_site: chrome_samesite(samesite),
                priority: CookiePriority::Medium,
                creation_time: now,
                last_access_time: now,
                host_only,
            };
            cookies.push(cookie);
        }

        Ok(cookies)
    }

    fn read_chromium_cookies_v2(&self, path: &PathBuf) -> CookieResult<Vec<CanonicalCookie>> {
        use rusqlite::{Connection, OpenFlags};

        let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

        let mut stmt = conn.prepare(
            "SELECT host_key, name, value, encrypted_value, path, expires_utc, is_secure, is_httponly, samesite
             FROM cookies",
        )?;

        let mut cookies = Vec::new();
        let now = OffsetDateTime::now_utc();

        let mut rows = stmt.query([])?;

        while let Some(row) = rows.next()? {
            let host_key: String = row.get(0).unwrap_or_default();
            let name: String = row.get(1).unwrap_or_default();
            let value: String = row.get(2).unwrap_or_default();
            let encrypted_value: Vec<u8> = row.get(3).unwrap_or_default();
            let path: String = row.get(4).unwrap_or_default();
            let expires_utc: i64 = row.get(5).unwrap_or(0);
            let is_secure: i32 = row.get(6).unwrap_or(0);
            let is_httponly: i32 = row.get(7).unwrap_or(0);
            let samesite: i32 = row.get(8).unwrap_or(-1);

            // Apply domain filter
            if let Some(ref filter) = self.domain_filter {
                if !host_key.ends_with(filter) && !host_key.trim_start_matches('.').eq(filter) {
                    continue;
                }
            }

            // Determine the cookie value
            let cookie_value = if !value.is_empty() {
                value
            } else if !encrypted_value.is_empty() {
                oscrypt::decrypt_cookie(&encrypted_value)?
            } else {
                continue;
            };

            let host_only = !host_key.starts_with('.');
            let cookie = CanonicalCookie {
                name,
                value: cookie_value,
                domain: host_key,
                path,
                expiration_time: chrome_time_to_offset(expires_utc),
                secure: is_secure != 0,
                http_only: is_httponly != 0,
                same_site: chrome_samesite(samesite),
                priority: CookiePriority::Medium,
                creation_time: now,
                last_access_time: now,
                host_only,
            };
            cookies.push(cookie);
        }

        Ok(cookies)
    }

    fn read_firefox_cookies(&self, path: &PathBuf) -> Result<Vec<CanonicalCookie>, NetError> {
        use rusqlite::{Connection, OpenFlags};

        let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|_| NetError::FileNotFound)?;

        let mut stmt = conn
            .prepare(
                "SELECT host, name, value, path, expiry, isSecure, isHttpOnly, sameSite
             FROM moz_cookies",
            )
            .map_err(|_| NetError::InvalidResponse)?;

        let cookie_iter = stmt
            .query_map([], |row| {
                Ok(FirefoxCookieRow {
                    host: row.get(0)?,
                    name: row.get(1)?,
                    value: row.get(2)?,
                    path: row.get(3)?,
                    expiry: row.get(4)?,
                    is_secure: row.get(5)?,
                    is_http_only: row.get(6)?,
                    same_site: row.get(7)?,
                })
            })
            .map_err(|_| NetError::InvalidResponse)?;

        let mut cookies = Vec::new();
        let now = OffsetDateTime::now_utc();

        for row in cookie_iter.flatten() {
            let cookie = CanonicalCookie {
                name: row.name,
                value: row.value,
                domain: row.host.clone(),
                path: row.path,
                expiration_time: firefox_time_to_offset(row.expiry),
                secure: row.is_secure != 0,
                http_only: row.is_http_only != 0,
                same_site: firefox_samesite(row.same_site),
                priority: CookiePriority::Medium,
                creation_time: now,
                last_access_time: now,
                host_only: !row.host.starts_with('.'),
            };
            cookies.push(cookie);
        }

        Ok(cookies)
    }

    fn read_firefox_cookies_v2(&self, path: &PathBuf) -> CookieResult<Vec<CanonicalCookie>> {
        use rusqlite::{Connection, OpenFlags};

        let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

        let mut stmt = conn.prepare(
            "SELECT host, name, value, path, expiry, isSecure, isHttpOnly, sameSite
             FROM moz_cookies",
        )?;

        let mut cookies = Vec::new();
        let now = OffsetDateTime::now_utc();

        let mut rows = stmt.query([])?;

        while let Some(row) = rows.next()? {
            let host: String = row.get(0).unwrap_or_default();
            let name: String = row.get(1).unwrap_or_default();
            let value: String = row.get(2).unwrap_or_default();
            let path: String = row.get(3).unwrap_or_default();
            let expiry: i64 = row.get(4).unwrap_or(0);
            let is_secure: i32 = row.get(5).unwrap_or(0);
            let is_http_only: i32 = row.get(6).unwrap_or(0);
            let same_site: i32 = row.get(7).unwrap_or(0);

            // Apply domain filter
            if let Some(ref filter) = self.domain_filter {
                if !host.ends_with(filter) && !host.trim_start_matches('.').eq(filter) {
                    continue;
                }
            }

            let cookie = CanonicalCookie {
                name,
                value, // Firefox stores cookies in plaintext
                domain: host.clone(),
                path,
                expiration_time: firefox_time_to_offset(expiry),
                secure: is_secure != 0,
                http_only: is_http_only != 0,
                same_site: firefox_samesite(same_site),
                priority: CookiePriority::Medium,
                creation_time: now,
                last_access_time: now,
                host_only: !host.starts_with('.'),
            };
            cookies.push(cookie);
        }

        Ok(cookies)
    }
}

#[allow(dead_code)]
struct ChromeCookieRow {
    host_key: String,
    name: String,
    value: String,
    path: String,
    expires_utc: i64,
    is_secure: i32,
    is_httponly: i32,
    samesite: i32,
}

struct FirefoxCookieRow {
    host: String,
    name: String,
    value: String,
    path: String,
    expiry: i64,
    is_secure: i32,
    is_http_only: i32,
    same_site: i32,
}

/// Convert Chrome's WebKit timestamp to OffsetDateTime.
/// Chrome uses microseconds since 1601-01-01.
fn chrome_time_to_offset(timestamp: i64) -> Option<OffsetDateTime> {
    if timestamp == 0 {
        return None; // Session cookie
    }
    // Chrome epoch is 1601-01-01, Unix epoch is 1970-01-01
    // Difference: 11644473600 seconds
    let unix_micros = timestamp - 11_644_473_600_000_000;
    OffsetDateTime::from_unix_timestamp_nanos(unix_micros as i128 * 1000).ok()
}

/// Convert Firefox's Unix timestamp to OffsetDateTime.
fn firefox_time_to_offset(timestamp: i64) -> Option<OffsetDateTime> {
    if timestamp == 0 {
        return None;
    }
    OffsetDateTime::from_unix_timestamp(timestamp).ok()
}

fn chrome_samesite(value: i32) -> SameSite {
    match value {
        0 => SameSite::NoRestriction, // None
        1 => SameSite::Lax,
        2 => SameSite::Strict,
        _ => SameSite::Unspecified,
    }
}

fn firefox_samesite(value: i32) -> SameSite {
    match value {
        0 => SameSite::NoRestriction,
        1 => SameSite::Lax,
        2 => SameSite::Strict,
        _ => SameSite::Unspecified,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_enum() {
        let chrome = Browser::Chrome;
        let firefox = Browser::Firefox;
        assert_ne!(chrome, firefox);
    }

    #[test]
    fn test_reader_creation() {
        let reader = BrowserCookieReader::new(Browser::Chrome);
        assert_eq!(reader.browser, Browser::Chrome);
    }

    #[test]
    fn test_with_profile() {
        let reader = BrowserCookieReader::new(Browser::Chrome).with_profile("Profile 1");
        assert_eq!(reader.profile, Some("Profile 1".to_string()));
    }

    #[test]
    fn test_chrome_time_conversion() {
        // Test session cookie (0 timestamp)
        assert!(chrome_time_to_offset(0).is_none());

        // Test valid timestamp
        let ts = chrome_time_to_offset(13300000000000000);
        assert!(ts.is_some());
    }

    #[test]
    fn test_firefox_time_conversion() {
        assert!(firefox_time_to_offset(0).is_none());

        let ts = firefox_time_to_offset(1700000000);
        assert!(ts.is_some());
    }

    #[test]
    fn test_samesite_conversion() {
        assert_eq!(chrome_samesite(1), SameSite::Lax);
        assert_eq!(firefox_samesite(2), SameSite::Strict);
    }
}
