//! Platform-specific cookie decryption modules.
//!
//! This module provides platform-specific decryption for Chrome's v11 encryption
//! which uses system keyrings/credential managers to store the encryption key.
//!
//! ## Platform Support
//! - **Linux**: libsecret/GNOME Keyring via `secret-service` crate
//! - **macOS**: Keychain via `security-framework` crate
//! - **Windows**: DPAPI via `windows` crate

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;

use crate::cookies::error::CookieExtractionError;

/// Derive a 16-byte AES key from a password using PBKDF2-HMAC-SHA1.
///
/// This matches Chromium's key derivation in `os_crypt`.
pub fn derive_key(password: &[u8], iterations: u32) -> [u8; 16] {
    use boring::hash::MessageDigest;
    use boring::pkcs5::pbkdf2_hmac;

    let salt = b"saltysalt";
    let mut key = [0u8; 16];

    pbkdf2_hmac(
        password,
        salt,
        iterations as usize,
        MessageDigest::sha1(),
        &mut key,
    )
    .expect("PBKDF2 should not fail");

    key
}

/// Get the Chrome encryption key from the system keyring.
///
/// This is a convenience function that calls the appropriate platform-specific
/// implementation based on the current OS.
#[allow(unused_variables)]
pub fn get_chrome_key(application: &str) -> Result<Option<[u8; 16]>, CookieExtractionError> {
    #[cfg(target_os = "linux")]
    {
        linux::get_v11_key(application)
    }

    #[cfg(target_os = "macos")]
    {
        macos::get_keychain_key(application)
    }

    #[cfg(target_os = "windows")]
    {
        // Windows uses a different flow - key comes from Local State file
        Ok(None)
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Err(CookieExtractionError::PlatformNotSupported(
            "Keyring access not supported on this platform".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_key_v10() {
        // Test v10 key derivation (1 iteration) - same as Linux
        let key = derive_key(b"peanuts", 1);
        let expected: [u8; 16] = [
            0xfd, 0x62, 0x1f, 0xe5, 0xa2, 0xb4, 0x02, 0x53, 0x9d, 0xfa, 0x14, 0x7c, 0xa9, 0x27,
            0x27, 0x78,
        ];
        assert_eq!(key, expected);
    }

    #[test]
    fn test_derive_key_empty() {
        // Test empty password (fallback key)
        let key = derive_key(b"", 1);
        let expected: [u8; 16] = [
            0xd0, 0xd0, 0xec, 0x9c, 0x7d, 0x77, 0xd4, 0x3a, 0xc5, 0x41, 0x87, 0xfa, 0x48, 0x18,
            0xd1, 0x7f,
        ];
        assert_eq!(key, expected);
    }

    #[test]
    fn test_derive_key_macos_iterations() {
        // macOS uses 1003 iterations - key should be different
        let key_linux = derive_key(b"test_password", 1);
        let key_macos = derive_key(b"test_password", 1003);
        assert_ne!(key_linux, key_macos);
    }

    #[test]
    fn test_derive_key_different_passwords() {
        let key1 = derive_key(b"password1", 1);
        let key2 = derive_key(b"password2", 1);
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_derive_key_unicode() {
        // Unicode password should work
        let key = derive_key("пароль".as_bytes(), 1);
        assert_eq!(key.len(), 16);
    }

    #[test]
    fn test_derive_key_long_password() {
        // Long password should work
        let long_password = "a".repeat(1000);
        let key = derive_key(long_password.as_bytes(), 1);
        assert_eq!(key.len(), 16);
    }
}
