//! Windows DPAPI and AES-GCM decryption for Chrome cookies.
//!
//! On Windows, Chrome stores the AES key in `Local State` JSON file,
//! encrypted with DPAPI. The cookie values are then encrypted with
//! AES-256-GCM (not AES-CBC like Linux/macOS v10).
//!
//! ## Encryption Format (Windows v10)
//! - Prefix: "v10" (3 bytes)
//! - Nonce: 12 bytes
//! - Ciphertext + Tag: remaining bytes
//! - Algorithm: AES-256-GCM

use crate::cookies::error::CookieExtractionError;
use std::path::Path;

/// Get the Local State file path for a Chromium-based browser.
pub fn get_local_state_path(
    browser: &str,
    profile_dir: Option<&Path>,
) -> Option<std::path::PathBuf> {
    if let Some(dir) = profile_dir {
        // Use provided profile directory, go up one level to User Data
        return Some(dir.parent()?.join("Local State"));
    }

    #[cfg(target_os = "windows")]
    {
        let local_app_data = std::env::var("LOCALAPPDATA").ok()?;
        let browser_path = match browser.to_lowercase().as_str() {
            "chrome" | "google-chrome" => "Google/Chrome/User Data",
            "chromium" => "Chromium/User Data",
            "edge" | "microsoft-edge" => "Microsoft/Edge/User Data",
            "brave" | "brave-browser" => "BraveSoftware/Brave-Browser/User Data",
            "opera" => "Opera Software/Opera Stable",
            _ => "Google/Chrome/User Data",
        };
        Some(std::path::PathBuf::from(format!(
            "{}/{}/Local State",
            local_app_data, browser_path
        )))
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = browser;
        None
    }
}

/// Decrypt Chrome's v10 encrypted cookie value on Windows.
///
/// # Arguments
/// * `encrypted` - The encrypted value (with "v10" prefix)
/// * `key` - The 32-byte AES-256 key
///
/// # Returns
/// * `Ok(plaintext)` - Successfully decrypted
/// * `Err(...)` - Decryption failed
#[cfg(target_os = "windows")]
pub fn decrypt_v10_windows(
    encrypted: &[u8],
    key: &[u8; 32],
) -> Result<String, CookieExtractionError> {
    use aes_gcm::{
        aead::{Aead, KeyInit},
        Aes256Gcm, Nonce,
    };

    const V10_PREFIX: &[u8] = b"v10";
    const NONCE_LEN: usize = 12;

    if !encrypted.starts_with(V10_PREFIX) {
        return Err(CookieExtractionError::DecryptionFailed(
            "Not a v10 encrypted value".into(),
        ));
    }

    let data = &encrypted[V10_PREFIX.len()..];
    if data.len() < NONCE_LEN {
        return Err(CookieExtractionError::DecryptionFailed(
            "Data too short".into(),
        ));
    }

    let nonce = Nonce::from_slice(&data[..NONCE_LEN]);
    let ciphertext = &data[NONCE_LEN..];

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| CookieExtractionError::DecryptionFailed("Invalid key".into()))?;

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| CookieExtractionError::DecryptionFailed("AES-GCM decryption failed".into()))?;

    String::from_utf8(plaintext)
        .map_err(|_| CookieExtractionError::InvalidData("Invalid UTF-8 in decrypted value".into()))
}

/// Get Chrome's encryption key from Local State file using DPAPI.
#[cfg(target_os = "windows")]
pub fn get_dpapi_key(local_state_path: &Path) -> Result<[u8; 32], CookieExtractionError> {
    use base64::Engine;
    use windows::Win32::Security::Cryptography::{CryptUnprotectData, CRYPT_INTEGER_BLOB};

    // Read Local State JSON
    let local_state = std::fs::read_to_string(local_state_path)?;
    let json: serde_json::Value = serde_json::from_str(&local_state)
        .map_err(|_| CookieExtractionError::InvalidData("Invalid Local State JSON".into()))?;

    // Extract encrypted_key from os_crypt section
    let encrypted_key_b64 = json["os_crypt"]["encrypted_key"].as_str().ok_or_else(|| {
        CookieExtractionError::InvalidData("No encrypted_key in Local State".into())
    })?;

    // Base64 decode
    let encrypted_key = base64::engine::general_purpose::STANDARD
        .decode(encrypted_key_b64)
        .map_err(|_| {
            CookieExtractionError::InvalidData("Invalid base64 in encrypted_key".into())
        })?;

    // Strip "DPAPI" prefix (5 bytes)
    const DPAPI_PREFIX: &[u8] = b"DPAPI";
    if !encrypted_key.starts_with(DPAPI_PREFIX) {
        return Err(CookieExtractionError::InvalidData(
            "Missing DPAPI prefix".into(),
        ));
    }
    let dpapi_data = &encrypted_key[DPAPI_PREFIX.len()..];

    // Decrypt with DPAPI
    let mut blob_in = CRYPT_INTEGER_BLOB {
        cbData: dpapi_data.len() as u32,
        pbData: dpapi_data.as_ptr() as *mut u8,
    };
    let mut blob_out = CRYPT_INTEGER_BLOB::default();

    unsafe {
        CryptUnprotectData(&mut blob_in, None, None, None, None, 0, &mut blob_out).map_err(
            |_| CookieExtractionError::DecryptionFailed("DPAPI decryption failed".into()),
        )?;

        if blob_out.cbData != 32 {
            return Err(CookieExtractionError::DecryptionFailed(
                "Unexpected key length from DPAPI".into(),
            ));
        }

        let mut key = [0u8; 32];
        std::ptr::copy_nonoverlapping(blob_out.pbData, key.as_mut_ptr(), 32);

        // Free the memory allocated by DPAPI
        windows::Win32::System::Memory::LocalFree(windows::Win32::Foundation::HLOCAL(
            blob_out.pbData as *mut _,
        ));

        Ok(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_state_path() {
        // Just test the path construction logic
        let path = get_local_state_path("chrome", None);
        // On non-Windows, this returns None
        #[cfg(not(target_os = "windows"))]
        assert!(path.is_none());
    }
}
