//! Browser cookie extraction from Chrome/Firefox SQLite databases.
//!
//! Reads cookies from local browser databases for session reuse.
//! Note: Encrypted cookies (v10/v11 on Windows/macOS) require
//! platform-specific decryption (DPAPI/Keychain) not yet implemented.

use crate::base::neterror::NetError;
use crate::cookies::canonical_cookie::{CanonicalCookie, CookiePriority, SameSite};
use std::path::PathBuf;
use time::OffsetDateTime;

/// Supported browsers for cookie extraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Browser {
    Chrome,
    Firefox,
}

/// Reader for browser cookie databases.
pub struct BrowserCookieReader {
    browser: Browser,
    profile: Option<String>,
}

impl BrowserCookieReader {
    /// Create a new reader for the specified browser.
    pub fn new(browser: Browser) -> Self {
        Self { browser, profile: None }
    }

    /// Use a specific profile (default: "Default" for Chrome, first profile for Firefox).
    pub fn with_profile(mut self, profile: impl Into<String>) -> Self {
        self.profile = Some(profile.into());
        self
    }

    /// Get the path to the browser's cookie database.
    pub fn get_db_path(&self) -> Option<PathBuf> {
        match self.browser {
            Browser::Chrome => self.chrome_cookie_path(),
            Browser::Firefox => self.firefox_cookie_path(),
        }
    }

    fn chrome_cookie_path(&self) -> Option<PathBuf> {
        let profile = self.profile.as_deref().unwrap_or("Default");

        #[cfg(target_os = "linux")]
        {
            let home = std::env::var("HOME").ok()?;
            Some(PathBuf::from(format!("{}/.config/google-chrome/{}/Cookies", home, profile)))
        }

        #[cfg(target_os = "macos")]
        {
            let home = std::env::var("HOME").ok()?;
            Some(PathBuf::from(format!(
                "{}/Library/Application Support/Google/Chrome/{}/Cookies",
                home, profile
            )))
        }

        #[cfg(target_os = "windows")]
        {
            let local_app_data = std::env::var("LOCALAPPDATA").ok()?;
            Some(PathBuf::from(format!(
                "{}/Google/Chrome/User Data/{}/Network/Cookies",
                local_app_data, profile
            )))
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            None
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
            let firefox_dir =
                PathBuf::from(format!("{}/Library/Application Support/Firefox/Profiles", home));

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
            Browser::Chrome => self.read_chrome_cookies(&db_path),
            Browser::Firefox => self.read_firefox_cookies(&db_path),
        }
    }

    fn read_chrome_cookies(&self, path: &PathBuf) -> Result<Vec<CanonicalCookie>, NetError> {
        use rusqlite::{Connection, OpenFlags};

        let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|_| NetError::FileNotFound)?;

        let mut stmt = conn
            .prepare(
                "SELECT host_key, name, value, path, expires_utc, is_secure, is_httponly, samesite
             FROM cookies",
            )
            .map_err(|_| NetError::InvalidResponse)?;

        let cookie_iter = stmt
            .query_map([], |row| {
                Ok(ChromeCookieRow {
                    host_key: row.get(0)?,
                    name: row.get(1)?,
                    value: row.get(2)?,
                    path: row.get(3)?,
                    expires_utc: row.get(4)?,
                    is_secure: row.get(5)?,
                    is_httponly: row.get(6)?,
                    samesite: row.get(7)?,
                })
            })
            .map_err(|_| NetError::InvalidResponse)?;

        let mut cookies = Vec::new();
        let now = OffsetDateTime::now_utc();

        for cookie_result in cookie_iter {
            if let Ok(row) = cookie_result {
                // Skip encrypted cookies (empty value means encrypted)
                if row.value.is_empty() {
                    continue;
                }

                let host_only = !row.host_key.starts_with('.');
                let cookie = CanonicalCookie {
                    name: row.name,
                    value: row.value,
                    domain: row.host_key,
                    path: row.path,
                    expiration_time: chrome_time_to_offset(row.expires_utc),
                    secure: row.is_secure != 0,
                    http_only: row.is_httponly != 0,
                    same_site: chrome_samesite(row.samesite),
                    priority: CookiePriority::Medium,
                    creation_time: now,
                    last_access_time: now,
                    host_only,
                };
                cookies.push(cookie);
            }
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

        for cookie_result in cookie_iter {
            if let Ok(row) = cookie_result {
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
        }

        Ok(cookies)
    }
}

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
    let unix_micros = timestamp - 11644473600_000_000;
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
