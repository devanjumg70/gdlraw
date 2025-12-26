//! macOS Keychain access for Chrome cookie decryption.
//!
//! Uses the Security Framework to retrieve Chrome's encryption password
//! from the macOS Keychain, then derives the AES key.
//!
//! ## Chrome's Keychain Entry
//! - Service: "Chrome Safe Storage" (or browser variant)
//! - Account: "Chrome" (or browser variant)
//! - Key derivation: PBKDF2-HMAC-SHA1 with 1003 iterations

use crate::base::neterror::NetError;

/// Keychain service names for each browser.
pub fn browser_keychain_service(browser: &str) -> &'static str {
    match browser.to_lowercase().as_str() {
        "chrome" | "google-chrome" => "Chrome Safe Storage",
        "chromium" => "Chromium Safe Storage",
        "edge" | "microsoft-edge" => "Microsoft Edge Safe Storage",
        "brave" | "brave-browser" => "Brave Safe Storage",
        "opera" => "Opera Safe Storage",
        _ => "Chrome Safe Storage",
    }
}

/// Keychain account names for each browser.
pub fn browser_keychain_account(browser: &str) -> &'static str {
    match browser.to_lowercase().as_str() {
        "chrome" | "google-chrome" => "Chrome",
        "chromium" => "Chromium",
        "edge" | "microsoft-edge" => "Microsoft Edge",
        "brave" | "brave-browser" => "Brave",
        "opera" => "Opera",
        _ => "Chrome",
    }
}

/// Get the encryption key from macOS Keychain.
///
/// # Arguments
/// * `application` - The application/browser name
///
/// # Returns
/// * `Ok(Some(key))` - Successfully retrieved and derived the key
/// * `Ok(None)` - Keychain entry not found
/// * `Err(...)` - Keychain access denied
#[cfg(target_os = "macos")]
pub fn get_keychain_key(application: &str) -> Result<Option<[u8; 16]>, NetError> {
    use security_framework::passwords::get_generic_password;

    let service = browser_keychain_service(application);
    let account = browser_keychain_account(application);

    // Try to get the password from Keychain
    match get_generic_password(service, account) {
        Ok(password) => {
            // Derive key with 1003 iterations (macOS uses more iterations than Linux)
            let key = super::derive_key(&password, 1003);
            Ok(Some(key))
        }
        Err(e) if e.code() == -25300 => {
            // errSecItemNotFound - no password stored
            Ok(None)
        }
        Err(_) => Err(NetError::CookieKeyringUnavailable),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keychain_service_names() {
        assert_eq!(browser_keychain_service("chrome"), "Chrome Safe Storage");
        assert_eq!(browser_keychain_service("brave"), "Brave Safe Storage");
        assert_eq!(
            browser_keychain_service("edge"),
            "Microsoft Edge Safe Storage"
        );
    }

    #[test]
    fn test_keychain_account_names() {
        assert_eq!(browser_keychain_account("chrome"), "Chrome");
        assert_eq!(browser_keychain_account("chromium"), "Chromium");
    }
}
