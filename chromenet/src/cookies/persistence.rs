//! Cookie persistence - save and load cookies to/from disk.
//!
//! Provides JSON-based persistence for CookieMonster.

use crate::cookies::monster::CookieMonster;
use std::fs;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// Serializable representation of a cookie for persistence.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct PersistentCookie {
    name: String,
    value: String,
    domain: String,
    path: String,
    secure: bool,
    http_only: bool,
    host_only: bool,
    expires_unix_secs: Option<i64>,
}

/// Save cookies from a CookieMonster to a file.
///
/// # Example
/// ```ignore
/// persistence::save_cookies(&monster, "/path/to/cookies.json")?;
/// ```
pub fn save_cookies(monster: &CookieMonster, path: &Path) -> io::Result<()> {
    let mut all_cookies = Vec::new();

    // Iterate through all cookies
    for cookie in monster.iter_all_cookies() {
        let expires = cookie.expiration_time.map(|t| t.unix_timestamp());

        all_cookies.push(PersistentCookie {
            name: cookie.name,
            value: cookie.value,
            domain: cookie.domain,
            path: cookie.path,
            secure: cookie.secure,
            http_only: cookie.http_only,
            host_only: cookie.host_only,
            expires_unix_secs: expires,
        });
    }

    let json = serde_json::to_string_pretty(&all_cookies)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    fs::write(path, json)
}

/// Load cookies from a file into a new CookieMonster.
///
/// # Example
/// ```ignore
/// let monster = persistence::load_cookies("/path/to/cookies.json")?;
/// ```
pub fn load_cookies(path: &Path) -> io::Result<CookieMonster> {
    use crate::cookies::canonical_cookie::{CanonicalCookie, CookiePriority, SameSite};
    use time::OffsetDateTime;

    let json = fs::read_to_string(path)?;
    let persistent_cookies: Vec<PersistentCookie> =
        serde_json::from_str(&json).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let monster = CookieMonster::new();
    let now = OffsetDateTime::now_utc();

    for pc in persistent_cookies {
        // Skip expired cookies
        if let Some(expires_secs) = pc.expires_unix_secs {
            if let Ok(expires) = OffsetDateTime::from_unix_timestamp(expires_secs) {
                if expires < now {
                    continue; // Skip expired
                }
            }
        }

        let expiration_time =
            pc.expires_unix_secs.and_then(|s| OffsetDateTime::from_unix_timestamp(s).ok());

        let cookie = CanonicalCookie {
            name: pc.name,
            value: pc.value,
            domain: pc.domain.clone(),
            path: pc.path,
            creation_time: now,
            expiration_time,
            last_access_time: now,
            secure: pc.secure,
            http_only: pc.http_only,
            host_only: pc.host_only,
            same_site: SameSite::Lax,
            priority: CookiePriority::Medium,
        };

        monster.set_canonical_cookie(cookie);
    }

    Ok(monster)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_save_load_roundtrip() {
        use crate::cookies::canonical_cookie::{CanonicalCookie, CookiePriority, SameSite};
        use time::OffsetDateTime;

        let monster = CookieMonster::new();
        let now = OffsetDateTime::now_utc();

        // Add a test cookie
        monster.set_canonical_cookie(CanonicalCookie {
            name: "session".to_string(),
            value: "abc123".to_string(),
            domain: "example.com".to_string(),
            path: "/".to_string(),
            creation_time: now,
            expiration_time: None,
            last_access_time: now,
            secure: true,
            http_only: true,
            host_only: false,
            same_site: SameSite::Lax,
            priority: CookiePriority::Medium,
        });

        // Save to temp file
        let dir = tempdir().unwrap();
        let path = dir.path().join("cookies.json");

        save_cookies(&monster, &path).unwrap();

        // Load back
        let loaded = load_cookies(&path).unwrap();

        // Verify
        assert_eq!(loaded.total_cookie_count(), 1);

        // Get cookie for example.com
        let url = url::Url::parse("https://example.com/").unwrap();
        let cookies = loaded.get_cookies_for_url(&url);
        assert_eq!(cookies.len(), 1);
        assert_eq!(cookies[0].name, "session");
        assert_eq!(cookies[0].value, "abc123");
    }
}
