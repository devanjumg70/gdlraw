//! Cookie extraction error types.
//!
//! # Deprecated
//!
//! This module is deprecated. Use [`crate::base::neterror::NetError`] instead.
//! Cookie extraction errors have been unified into `NetError`.

use crate::base::neterror::NetError;

/// Cookie extraction error type.
///
/// # Deprecated
///
/// This type is deprecated. Use [`NetError`] instead.
/// The following mappings apply:
///
/// | Old | New |
/// |-----|-----|
/// | `CookieExtractionError::BrowserNotFound` | `NetError::BrowserNotFound` |
/// | `CookieExtractionError::DatabaseNotFound` | `NetError::CookieDbNotFound` |
/// | `CookieExtractionError::DecryptionFailed` | `NetError::CookieDecryptionFailed` |
/// | `CookieExtractionError::DatabaseLocked` | `NetError::CookieDatabaseLocked` |
/// | `CookieExtractionError::UnsupportedVersion` | `NetError::CookieUnsupportedVersion` |
/// | `CookieExtractionError::PlatformNotSupported` | `NetError::CookiePlatformNotSupported` |
/// | `CookieExtractionError::ProfileNotFound` | `NetError::CookieProfileNotFound` |
/// | `CookieExtractionError::KeyringUnavailable` | `NetError::CookieKeyringUnavailable` |
/// | `CookieExtractionError::InvalidData` | `NetError::CookieInvalidData` |
/// | `CookieExtractionError::Database` | `NetError::CookieDatabaseError` |
#[deprecated(since = "0.2.0", note = "Use crate::base::neterror::NetError instead")]
pub type CookieExtractionError = NetError;

/// Result type alias for cookie extraction operations.
#[deprecated(since = "0.2.0", note = "Use Result<T, NetError> instead")]
pub type CookieResult<T> = Result<T, NetError>;

// Helper functions for creating cookie errors with the new types

/// Create a browser not found error.
#[deprecated(since = "0.2.0", note = "Use NetError::browser_not_found instead")]
pub fn browser_not_found(browser: impl Into<String>) -> NetError {
    NetError::browser_not_found(browser)
}

/// Create a database not found error.
#[deprecated(since = "0.2.0", note = "Use NetError::cookie_db_not_found instead")]
pub fn database_not_found(path: impl Into<String>) -> NetError {
    NetError::cookie_db_not_found(path)
}

/// Create a decryption failed error.
#[deprecated(
    since = "0.2.0",
    note = "Use NetError::cookie_decryption_failed instead"
)]
pub fn decryption_failed(browser: impl Into<String>, reason: impl Into<String>) -> NetError {
    NetError::cookie_decryption_failed(browser, reason)
}

/// Create an invalid data error.
#[deprecated(since = "0.2.0", note = "Use NetError::cookie_invalid_data instead")]
pub fn invalid_data(reason: impl Into<String>) -> NetError {
    NetError::cookie_invalid_data(reason)
}

// Conversion from rusqlite errors
impl From<rusqlite::Error> for NetError {
    fn from(err: rusqlite::Error) -> Self {
        match err {
            rusqlite::Error::SqliteFailure(e, _)
                if e.code == rusqlite::ffi::ErrorCode::DatabaseBusy
                    || e.code == rusqlite::ffi::ErrorCode::DatabaseLocked =>
            {
                NetError::CookieDatabaseLocked
            }
            _ => NetError::CookieDatabaseError {
                message: err.to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(deprecated)]
    fn test_type_alias_works() {
        let err: CookieExtractionError = NetError::CookieDatabaseLocked;
        assert!(matches!(err, NetError::CookieDatabaseLocked));
    }
}
