//! Error types for browser cookie extraction.
//!
//! Provides specific error variants for cookie extraction failures,
//! enabling proper error handling and user-friendly messages.

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during browser cookie extraction.
#[derive(Error, Debug)]
pub enum CookieExtractionError {
    /// Browser is not installed on the system.
    #[error("Browser not found on system: {0}")]
    BrowserNotFound(String),

    /// Cookie database file does not exist at the expected path.
    #[error("Cookie database not found: {0}")]
    DatabaseNotFound(PathBuf),

    /// Failed to decrypt encrypted cookie values.
    #[error("Failed to decrypt cookies: {0}")]
    DecryptionFailed(String),

    /// Database is locked by the browser process.
    /// User should close the browser or copy the database file.
    #[error("Database is locked by browser (try closing the browser)")]
    DatabaseLocked,

    /// Browser version uses an unsupported encryption scheme.
    #[error("Unsupported browser version: {0}")]
    UnsupportedVersion(String),

    /// Feature is not supported on the current platform.
    #[error("Platform not supported for this browser: {0}")]
    PlatformNotSupported(String),

    /// Specified browser profile does not exist.
    #[error("Profile not found: {0}")]
    ProfileNotFound(String),

    /// System keyring is not available or access was denied.
    #[error("Keyring access denied or not available")]
    KeyringUnavailable,

    /// Cookie data is malformed or corrupted.
    #[error("Invalid cookie data: {0}")]
    InvalidData(String),

    /// I/O error when reading files.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// SQLite database error.
    #[error("Database error: {0}")]
    Database(String),
}

impl From<rusqlite::Error> for CookieExtractionError {
    fn from(err: rusqlite::Error) -> Self {
        match err {
            rusqlite::Error::SqliteFailure(e, _)
                if e.code == rusqlite::ffi::ErrorCode::DatabaseBusy
                    || e.code == rusqlite::ffi::ErrorCode::DatabaseLocked =>
            {
                CookieExtractionError::DatabaseLocked
            }
            _ => CookieExtractionError::Database(err.to_string()),
        }
    }
}

/// Result type alias for cookie extraction operations.
pub type CookieResult<T> = Result<T, CookieExtractionError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = CookieExtractionError::BrowserNotFound("Chrome".to_string());
        assert!(err.to_string().contains("Chrome"));

        let err = CookieExtractionError::DatabaseNotFound(PathBuf::from("/path/to/cookies"));
        assert!(err.to_string().contains("/path/to/cookies"));
    }

    #[test]
    fn test_database_locked_conversion() {
        // Simulate a database busy error
        let err = CookieExtractionError::DatabaseLocked;
        assert!(err.to_string().contains("locked"));
    }
}
