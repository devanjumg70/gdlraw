//! Safari binary cookies parser (`Cookies.binarycookies`).
//!
//! Safari stores cookies in a proprietary binary format on macOS.
//! This module parses the format to extract cookies.
//!
//! ## File Format
//! The file consists of:
//! 1. Header: "cook" magic + page count + page sizes
//! 2. Pages: Each page contains multiple cookies
//! 3. Footer: Checksum (ignored)
//!
//! ## References
//! - https://github.com/libyal/dtformats/blob/main/documentation/Safari%20Cookies.asciidoc
//! - https://github.com/als0052/BinaryCookieReader

use crate::base::neterror::NetError;
use crate::cookies::canonicalcookie::{CanonicalCookie, CookiePriority, SameSite};
use std::io::{Cursor, Read};
use time::OffsetDateTime;

/// Magic bytes at the start of a Safari binary cookies file.
const MAGIC: &[u8; 4] = b"cook";

/// Parse a Safari binary cookies file.
///
/// # Arguments
/// * `data` - The raw bytes of the Cookies.binarycookies file
///
/// # Returns
/// * `Ok(cookies)` - Successfully parsed cookies
/// * `Err(...)` - Parse error
pub fn parse_binary_cookies(data: &[u8]) -> Result<Vec<CanonicalCookie>, NetError> {
    if data.len() < 8 {
        return Err(NetError::cookie_invalid_data("File too small"));
    }

    // Check magic
    if &data[0..4] != MAGIC {
        return Err(NetError::cookie_invalid_data(
            "Invalid magic bytes (not a Safari cookies file)",
        ));
    }

    let mut cursor = Cursor::new(data);
    cursor.set_position(4);

    // Read number of pages (big-endian)
    let num_pages = read_u32_be(&mut cursor)?;

    // Read page sizes
    let mut page_sizes = Vec::with_capacity(num_pages as usize);
    for _ in 0..num_pages {
        page_sizes.push(read_u32_be(&mut cursor)?);
    }

    // Parse each page
    let mut all_cookies = Vec::new();
    for page_size in page_sizes {
        let page_start = cursor.position() as usize;
        let page_end = page_start + page_size as usize;

        if page_end > data.len() {
            return Err(NetError::cookie_invalid_data("Page extends beyond file"));
        }

        let page_data = &data[page_start..page_end];
        let cookies = parse_page(page_data)?;
        all_cookies.extend(cookies);

        cursor.set_position(page_end as u64);
    }

    Ok(all_cookies)
}

/// Parse a single page of cookies.
fn parse_page(data: &[u8]) -> Result<Vec<CanonicalCookie>, NetError> {
    if data.len() < 8 {
        return Err(NetError::cookie_invalid_data("Page too small"));
    }

    let mut cursor = Cursor::new(data);

    // Page header: 4 bytes (should be 0x00000100)
    let _header = read_u32_be(&mut cursor)?;

    // Number of cookies in this page
    let num_cookies = read_u32_le(&mut cursor)?;

    // Cookie offsets (little-endian)
    let mut cookie_offsets = Vec::with_capacity(num_cookies as usize);
    for _ in 0..num_cookies {
        cookie_offsets.push(read_u32_le(&mut cursor)?);
    }

    // Parse each cookie
    let mut cookies = Vec::new();
    for offset in cookie_offsets {
        let cookie = parse_cookie(&data[offset as usize..])?;
        cookies.push(cookie);
    }

    Ok(cookies)
}

/// Parse a single cookie record.
fn parse_cookie(data: &[u8]) -> Result<CanonicalCookie, NetError> {
    if data.len() < 48 {
        return Err(NetError::cookie_invalid_data("Cookie record too small"));
    }

    let mut cursor = Cursor::new(data);

    // Cookie size (4 bytes, little-endian)
    let _size = read_u32_le(&mut cursor)?;

    // Unknown (4 bytes)
    let _unknown1 = read_u32_le(&mut cursor)?;

    // Flags (4 bytes, little-endian)
    let flags = read_u32_le(&mut cursor)?;

    // Unknown (4 bytes)
    let _unknown2 = read_u32_le(&mut cursor)?;

    // URL offset (4 bytes)
    let url_offset = read_u32_le(&mut cursor)?;

    // Name offset (4 bytes)
    let name_offset = read_u32_le(&mut cursor)?;

    // Path offset (4 bytes)
    let path_offset = read_u32_le(&mut cursor)?;

    // Value offset (4 bytes)
    let value_offset = read_u32_le(&mut cursor)?;

    // Comment offset (4 bytes) - rarely used
    let _comment_offset = read_u32_le(&mut cursor)?;

    // End header (4 bytes)
    let _end_header = read_u32_le(&mut cursor)?;

    // Expiry date (8 bytes, double, Mac Absolute Time)
    let expiry_time = read_f64_le(&mut cursor)?;

    // Creation date (8 bytes, double)
    let creation_time = read_f64_le(&mut cursor)?;

    // Read strings
    let domain = read_null_terminated_string(data, url_offset as usize)?;
    let name = read_null_terminated_string(data, name_offset as usize)?;
    let path = read_null_terminated_string(data, path_offset as usize)?;
    let value = read_null_terminated_string(data, value_offset as usize)?;

    // Parse flags
    let secure = (flags & 0x01) != 0;
    let http_only = (flags & 0x04) != 0;
    let host_only = !domain.starts_with('.');

    // Convert Mac Absolute Time to OffsetDateTime
    let expiration = mac_absolute_time_to_offset(expiry_time);
    let creation = mac_absolute_time_to_offset(creation_time);

    Ok(CanonicalCookie {
        name,
        value,
        domain,
        path,
        expiration_time: expiration,
        secure,
        http_only,
        same_site: SameSite::Lax, // Safari doesn't store SameSite in binary format
        priority: CookiePriority::Medium,
        creation_time: creation.unwrap_or_else(OffsetDateTime::now_utc),
        last_access_time: creation.unwrap_or_else(OffsetDateTime::now_utc),
        host_only,
    })
}

/// Read a 32-bit unsigned integer in big-endian.
fn read_u32_be(cursor: &mut Cursor<&[u8]>) -> Result<u32, NetError> {
    let mut buf = [0u8; 4];
    cursor
        .read_exact(&mut buf)
        .map_err(|_| NetError::cookie_invalid_data("Unexpected EOF"))?;
    Ok(u32::from_be_bytes(buf))
}

/// Read a 32-bit unsigned integer in little-endian.
fn read_u32_le(cursor: &mut Cursor<&[u8]>) -> Result<u32, NetError> {
    let mut buf = [0u8; 4];
    cursor
        .read_exact(&mut buf)
        .map_err(|_| NetError::cookie_invalid_data("Unexpected EOF"))?;
    Ok(u32::from_le_bytes(buf))
}

/// Read a 64-bit float in little-endian.
fn read_f64_le(cursor: &mut Cursor<&[u8]>) -> Result<f64, NetError> {
    let mut buf = [0u8; 8];
    cursor
        .read_exact(&mut buf)
        .map_err(|_| NetError::cookie_invalid_data("Unexpected EOF"))?;
    Ok(f64::from_le_bytes(buf))
}

/// Read a null-terminated string from the data.
fn read_null_terminated_string(data: &[u8], offset: usize) -> Result<String, NetError> {
    if offset >= data.len() {
        return Err(NetError::cookie_invalid_data("String offset out of bounds"));
    }

    let slice = &data[offset..];
    let end = slice.iter().position(|&b| b == 0).unwrap_or(slice.len());
    String::from_utf8(slice[..end].to_vec())
        .map_err(|_| NetError::cookie_invalid_data("Invalid UTF-8 in string"))
}

/// Convert Mac Absolute Time to OffsetDateTime.
///
/// Mac Absolute Time is seconds since 2001-01-01 00:00:00 UTC.
fn mac_absolute_time_to_offset(timestamp: f64) -> Option<OffsetDateTime> {
    if timestamp <= 0.0 {
        return None;
    }

    // Mac epoch: 2001-01-01 00:00:00 UTC
    // Unix epoch: 1970-01-01 00:00:00 UTC
    // Difference: 978307200 seconds
    const MAC_TO_UNIX: i64 = 978_307_200;

    let unix_secs = (timestamp as i64) + MAC_TO_UNIX;
    OffsetDateTime::from_unix_timestamp(unix_secs).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magic_constant() {
        assert_eq!(MAGIC, b"cook");
    }

    #[test]
    fn test_invalid_magic() {
        let data = b"badm\x00\x00\x00\x00";
        let result = parse_binary_cookies(data);
        assert!(result.is_err());
    }

    #[test]
    fn test_file_too_small() {
        let data = b"cook";
        let result = parse_binary_cookies(data);
        assert!(result.is_err());
    }

    #[test]
    fn test_mac_absolute_time_conversion() {
        // Test a known timestamp - just verify the conversion works
        // Mac epoch is 2001-01-01, so 0.0 is 2001-01-01
        // 1.0 is 2001-01-01 + 1 second
        let mac_time = 1.0;
        let result = mac_absolute_time_to_offset(mac_time);
        assert!(result.is_some());
        let dt = result.unwrap();
        assert_eq!(dt.year(), 2001);
    }

    #[test]
    fn test_mac_absolute_time_zero() {
        let result = mac_absolute_time_to_offset(0.0);
        assert!(result.is_none());
    }

    #[test]
    fn test_mac_absolute_time_negative() {
        let result = mac_absolute_time_to_offset(-1.0);
        assert!(result.is_none());
    }

    #[test]
    fn test_read_null_terminated_string() {
        let data = b"hello\x00world";
        let result = read_null_terminated_string(data, 0);
        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    fn test_read_null_terminated_string_no_null() {
        let data = b"hello";
        let result = read_null_terminated_string(data, 0);
        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    fn test_empty_file_with_zero_pages() {
        // Valid header with 0 pages
        let mut data = Vec::new();
        data.extend_from_slice(b"cook");
        data.extend_from_slice(&0u32.to_be_bytes()); // 0 pages

        let result = parse_binary_cookies(&data);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
