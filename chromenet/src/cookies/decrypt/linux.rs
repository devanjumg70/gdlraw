//! Linux keyring access for Chrome v11 cookie decryption.
//!
//! Uses the Secret Service API (GNOME Keyring / KWallet) to retrieve
//! Chrome's encryption password, then derives the AES key.
//!
//! ## Chrome's Keyring Schema
//! - Schema name: `chrome_libsecret_os_crypt_password_v2`
//! - Attribute: `("application", "chrome")` (or browser variant)
//! - Label: "Chrome Safe Storage" or "Chromium Safe Storage"

use crate::cookies::error::CookieExtractionError;
use std::collections::HashMap;

/// Get the v11 encryption key from GNOME Keyring/Secret Service.
///
/// # Arguments
/// * `application` - The application name (e.g., "chrome", "chromium", "brave")
///
/// # Returns
/// * `Ok(Some(key))` - Successfully retrieved and derived the key
/// * `Ok(None)` - Keyring is available but no key found for this application
/// * `Err(...)` - Keyring is unavailable or access was denied
#[cfg(target_os = "linux")]
pub fn get_v11_key(application: &str) -> Result<Option<[u8; 16]>, CookieExtractionError> {
    // Use the blocking API for simplicity (no async runtime needed)
    use secret_service::blocking::SecretService;
    use secret_service::EncryptionType;

    // Connect to Secret Service
    let ss = SecretService::connect(EncryptionType::Dh)
        .map_err(|_| CookieExtractionError::KeyringUnavailable)?;

    // Search for Chrome's password using the application attribute
    let mut attributes = HashMap::new();
    attributes.insert("application", application);

    let search_result = ss
        .search_items(attributes)
        .map_err(|_| CookieExtractionError::KeyringUnavailable)?;

    // Check unlocked items first, then locked
    let item = search_result
        .unlocked
        .first()
        .or_else(|| search_result.locked.first());

    let Some(item) = item else {
        return Ok(None); // No key found for this application
    };

    // Unlock if needed
    if search_result.unlocked.is_empty() {
        item.unlock()
            .map_err(|_| CookieExtractionError::KeyringUnavailable)?;
    }

    // Get the secret (password)
    let secret = item
        .get_secret()
        .map_err(|_| CookieExtractionError::KeyringUnavailable)?;

    // Derive the AES key using PBKDF2 (1 iteration for Linux)
    let key = super::derive_key(&secret, 1);

    Ok(Some(key))
}

/// Get the application name for keyring lookup based on browser type.
pub fn browser_to_application(browser: &str) -> &'static str {
    match browser.to_lowercase().as_str() {
        "chrome" | "google-chrome" => "chrome",
        "chromium" => "chromium",
        "edge" | "microsoft-edge" => "chromium", // Edge uses chromium keyring
        "brave" | "brave-browser" => "brave",
        "opera" => "chromium", // Opera uses chromium keyring
        _ => "chrome",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_to_application() {
        assert_eq!(browser_to_application("chrome"), "chrome");
        assert_eq!(browser_to_application("Chromium"), "chromium");
        assert_eq!(browser_to_application("brave"), "brave");
        assert_eq!(browser_to_application("edge"), "chromium");
    }
}
