//! Chrome os_crypt compatible decryption.
//!
//! Decrypts Chrome's encrypted cookie values using the v10 algorithm.
//! Based on Chromium's `components/os_crypt/sync/os_crypt_linux.cc`.
//!
//! ## Encryption Versions
//! - **v10 (Linux)**: AES-CBC with PBKDF2-derived key from "peanuts"/"saltysalt"
//! - **v10 (macOS)**: AES-CBC with Keychain-derived key (1003 PBKDF2 iterations)
//! - **v10 (Windows)**: AES-GCM with DPAPI-protected key
//! - **v11 (Linux)**: AES-CBC with Keyring-derived key

use crate::base::neterror::NetError;
use crate::cookies::error::CookieExtractionError;

/// v10 prefix used by Chrome for encrypted values.
pub const V10_PREFIX: &[u8] = b"v10";
/// v11 prefix (requires keyring access).
pub const V11_PREFIX: &[u8] = b"v11";

/// Pre-computed v10 decryption key for Linux.
/// This is PBKDF2-HMAC-SHA1(password="peanuts", salt="saltysalt", iterations=1, dklen=16)
const V10_KEY: [u8; 16] = [
    0xfd, 0x62, 0x1f, 0xe5, 0xa2, 0xb4, 0x02, 0x53, 0x9d, 0xfa, 0x14, 0x7c, 0xa9, 0x27, 0x27, 0x78,
];

/// Pre-computed empty key for fallback decryption.
/// PBKDF2-HMAC-SHA1(password="", salt="saltysalt", iterations=1, dklen=16)
/// See: https://crbug.com/40055416
const EMPTY_KEY: [u8; 16] = [
    0xd0, 0xd0, 0xec, 0x9c, 0x7d, 0x77, 0xd4, 0x3a, 0xc5, 0x41, 0x87, 0xfa, 0x48, 0x18, 0xd1, 0x7f,
];

/// IV used by Chrome v10: 16 space characters (0x20).
const V10_IV: [u8; 16] = [0x20; 16];

/// PBKDF2 salt used by Chrome.
pub const CHROME_SALT: &[u8] = b"saltysalt";

/// Decrypt Chrome v10 encrypted data (Linux) with fallback to empty key.
///
/// The input should be the raw `encrypted_value` from the Cookies SQLite database.
/// Returns None if decryption fails or the data is not v10 encrypted.
///
/// This function automatically tries the empty key fallback if the normal key fails,
/// as per Chromium bug https://crbug.com/40055416.
pub fn decrypt_v10(encrypted: &[u8]) -> Option<String> {
    // Check for v10 prefix
    if !encrypted.starts_with(V10_PREFIX) {
        return None;
    }

    // Skip the "v10" prefix
    let ciphertext = &encrypted[V10_PREFIX.len()..];

    if ciphertext.is_empty() {
        return Some(String::new());
    }

    // Try with normal v10 key first
    if let Some(plaintext) = decrypt_aes_cbc(&V10_KEY, &V10_IV, ciphertext) {
        if let Ok(s) = String::from_utf8(plaintext) {
            return Some(s);
        }
    }

    // Fallback to empty key (Chromium bug workaround)
    if let Some(plaintext) = decrypt_aes_cbc(&EMPTY_KEY, &V10_IV, ciphertext) {
        if let Ok(s) = String::from_utf8(plaintext) {
            return Some(s);
        }
    }

    None
}

/// Decrypt v10 encrypted data with a custom key.
///
/// Used for macOS (Keychain-derived key) and Linux v11 (Keyring-derived key).
/// The key should already be derived using PBKDF2-HMAC-SHA1.
pub fn decrypt_v10_with_key(encrypted: &[u8], key: &[u8; 16]) -> Option<String> {
    // Check for v10 or v11 prefix
    let ciphertext = if encrypted.starts_with(V10_PREFIX) {
        &encrypted[V10_PREFIX.len()..]
    } else if encrypted.starts_with(V11_PREFIX) {
        &encrypted[V11_PREFIX.len()..]
    } else {
        return None;
    };

    if ciphertext.is_empty() {
        return Some(String::new());
    }

    let plaintext = decrypt_aes_cbc(key, &V10_IV, ciphertext)?;
    String::from_utf8(plaintext).ok()
}

/// Decrypt AES-CBC encrypted data with PKCS7 padding.
fn decrypt_aes_cbc(key: &[u8; 16], iv: &[u8; 16], data: &[u8]) -> Option<Vec<u8>> {
    use boring::symm::{Cipher, Crypter, Mode};

    // Data must be a multiple of block size
    if data.is_empty() || data.len() % 16 != 0 {
        return None;
    }

    let cipher = Cipher::aes_128_cbc();
    let mut crypter = Crypter::new(cipher, Mode::Decrypt, key, Some(iv)).ok()?;
    crypter.pad(true); // PKCS7 padding

    let mut plaintext = vec![0u8; data.len() + 16];
    let count = crypter.update(data, &mut plaintext).ok()?;
    let rest = crypter.finalize(&mut plaintext[count..]).ok()?;
    plaintext.truncate(count + rest);

    Some(plaintext)
}

/// Check if encrypted data has a known Chrome encryption prefix.
pub fn is_encrypted(data: &[u8]) -> bool {
    data.starts_with(V10_PREFIX) || data.starts_with(V11_PREFIX)
}

/// Get the encryption version from the prefix.
pub fn encryption_version(data: &[u8]) -> Option<u8> {
    if data.starts_with(V10_PREFIX) {
        Some(10)
    } else if data.starts_with(V11_PREFIX) {
        Some(11)
    } else {
        None
    }
}

/// Attempt to decrypt any Chrome-encrypted value (Linux v10 only).
///
/// For v11 (keyring) or other platforms, use `decrypt_with_key`.
pub fn decrypt(encrypted: &[u8]) -> Result<String, NetError> {
    if encrypted.starts_with(V10_PREFIX) {
        decrypt_v10(encrypted).ok_or(NetError::InvalidResponse)
    } else if encrypted.starts_with(V11_PREFIX) {
        // v11 requires keyring access - not yet implemented
        Err(NetError::NotImplemented)
    } else if encrypted.is_empty() {
        Ok(String::new())
    } else {
        // Try as plain text (unencrypted)
        String::from_utf8(encrypted.to_vec()).map_err(|_| NetError::InvalidUtf8)
    }
}

/// Attempt to decrypt with better error handling.
pub fn decrypt_cookie(encrypted: &[u8]) -> Result<String, CookieExtractionError> {
    if encrypted.starts_with(V10_PREFIX) {
        decrypt_v10(encrypted)
            .ok_or_else(|| CookieExtractionError::DecryptionFailed("v10 decryption failed".into()))
    } else if encrypted.starts_with(V11_PREFIX) {
        Err(CookieExtractionError::DecryptionFailed(
            "v11 requires keyring access (not yet implemented)".into(),
        ))
    } else if encrypted.is_empty() {
        Ok(String::new())
    } else {
        // Try as plain text (unencrypted)
        String::from_utf8(encrypted.to_vec())
            .map_err(|_| CookieExtractionError::InvalidData("invalid UTF-8 in cookie value".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v10_key_constant() {
        // Verify our pre-computed key matches Chromium's
        assert_eq!(V10_KEY.len(), 16);
        assert_eq!(V10_KEY[0], 0xfd);
        assert_eq!(V10_KEY[15], 0x78);
    }

    #[test]
    fn test_is_encrypted() {
        assert!(is_encrypted(b"v10abc"));
        assert!(is_encrypted(b"v11xyz"));
        assert!(!is_encrypted(b"plain"));
        assert!(!is_encrypted(b""));
    }

    #[test]
    fn test_encryption_version() {
        assert_eq!(encryption_version(b"v10abc"), Some(10));
        assert_eq!(encryption_version(b"v11xyz"), Some(11));
        assert_eq!(encryption_version(b"plain"), None);
    }

    #[test]
    fn test_decrypt_empty() {
        let result = decrypt(b"");
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_decrypt_plaintext() {
        let result = decrypt(b"plain_cookie_value");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "plain_cookie_value");
    }

    #[test]
    fn test_decrypt_v11_not_implemented() {
        let result = decrypt(b"v11someciphertext");
        assert!(matches!(result, Err(NetError::NotImplemented)));
    }
}
