use crate::base::neterror::NetError;
use http::header::{HeaderName, HeaderValue};
use http::HeaderMap;
use std::str::FromStr;

/// A header map that strictly preserves insertion order.
/// Used to construct requests with specific fingerprinting characteristics.
#[derive(Debug, Clone, Default)]
pub struct OrderedHeaderMap {
    headers: Vec<(HeaderName, HeaderValue)>,
}

impl OrderedHeaderMap {
    pub fn new() -> Self {
        Self {
            headers: Vec::new(),
        }
    }

    pub fn insert(&mut self, name: &str, value: &str) -> Result<(), NetError> {
        let name_header = HeaderName::from_str(name).map_err(|_| NetError::InvalidHeader)?;
        let value_header = HeaderValue::from_str(value).map_err(|_| NetError::InvalidHeader)?;

        // Chromium behavior: Update in place if exists (case-insensitive key match), else append.
        // Since HeaderName is already lowercase, simple equality works.
        if let Some((_, v)) = self.headers.iter_mut().find(|(n, _)| *n == name_header) {
            *v = value_header;
        } else {
            self.headers.push((name_header, value_header));
        }
        Ok(())
    }

    pub fn remove(&mut self, name: &str) {
        // Prepare lowercase comparison
        // But HeaderName::from_str handles it?
        // If name matches "Foo", HeaderName("foo").
        if let Ok(target) = HeaderName::from_str(name) {
            self.headers.retain(|(n, _)| n != target);
        }
    }

    pub fn get(&self, name: &str) -> Option<&HeaderValue> {
        if let Ok(target) = HeaderName::from_str(name) {
            self.headers
                .iter()
                .find(|(n, _)| n == target)
                .map(|(_, v)| v)
        } else {
            None
        }
    }

    /// Consumes the map and returns a standard http::HeaderMap.
    /// Note: http::HeaderMap preserves insertion order.
    pub fn to_header_map(self) -> HeaderMap {
        let mut map = HeaderMap::with_capacity(self.headers.len());
        for (name, value) in self.headers {
            map.append(name, value);
        }
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut headers = OrderedHeaderMap::new();
        headers.insert("Content-Type", "application/json").unwrap();
        assert_eq!(
            headers.get("Content-Type").unwrap().to_str().unwrap(),
            "application/json"
        );
    }

    #[test]
    fn test_case_insensitive_get() {
        let mut headers = OrderedHeaderMap::new();
        headers.insert("ACCEPT", "text/html").unwrap();
        assert!(headers.get("accept").is_some());
        assert!(headers.get("Accept").is_some());
    }

    #[test]
    fn test_update_existing_header() {
        let mut headers = OrderedHeaderMap::new();
        headers.insert("Host", "example.com").unwrap();
        headers.insert("Host", "updated.com").unwrap();
        assert_eq!(
            headers.get("Host").unwrap().to_str().unwrap(),
            "updated.com"
        );
        // Should still be only one entry
        assert_eq!(headers.to_header_map().len(), 1);
    }

    #[test]
    fn test_remove_header() {
        let mut headers = OrderedHeaderMap::new();
        headers.insert("X-Custom", "value").unwrap();
        headers.remove("X-Custom");
        assert!(headers.get("X-Custom").is_none());
    }

    #[test]
    fn test_preserves_insertion_order() {
        let mut headers = OrderedHeaderMap::new();
        headers.insert("Host", "example.com").unwrap();
        headers.insert("Accept", "text/html").unwrap();
        headers.insert("User-Agent", "test").unwrap();

        let map = headers.to_header_map();
        let names: Vec<_> = map.keys().collect();

        // First header should be Host
        assert_eq!(names[0].as_str(), "host");
    }

    #[test]
    fn test_invalid_header_name() {
        let mut headers = OrderedHeaderMap::new();
        let result = headers.insert("Invalid Header", "value");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_header_value() {
        let mut headers = OrderedHeaderMap::new();
        let result = headers.insert("Valid", "invalid\nvalue");
        assert!(result.is_err());
    }

    #[test]
    fn test_default_is_empty() {
        let headers = OrderedHeaderMap::default();
        assert!(headers.get("Any").is_none());
    }

    #[test]
    fn test_clone() {
        let mut headers = OrderedHeaderMap::new();
        headers.insert("Test", "value").unwrap();
        let cloned = headers.clone();
        assert!(cloned.get("Test").is_some());
    }
}

/// Case-sensitive header map for HTTP/1.1.
///
/// Preserves original header casing for fingerprinting.
/// HTTP/1.1 headers are case-insensitive per spec, but some
/// servers and fingerprinting detectors check exact casing.
#[derive(Debug, Clone, Default)]
pub struct CaseSensitiveHeaders {
    /// Headers as (original_name, value) pairs
    headers: Vec<(String, String)>,
}

impl CaseSensitiveHeaders {
    /// Create a new case-sensitive header map.
    pub fn new() -> Self {
        Self {
            headers: Vec::new(),
        }
    }

    /// Insert header with preserved casing.
    pub fn insert(&mut self, name: impl Into<String>, value: impl Into<String>) {
        let name = name.into();
        let value = value.into();

        // Update existing (case-insensitive match)
        if let Some((_, v)) = self
            .headers
            .iter_mut()
            .find(|(n, _)| n.eq_ignore_ascii_case(&name))
        {
            *v = value;
        } else {
            self.headers.push((name, value));
        }
    }

    /// Get header value (case-insensitive lookup).
    pub fn get(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    /// Convert to title case (e.g., "content-type" -> "Content-Type").
    pub fn as_title_case(&self) -> impl Iterator<Item = (String, &str)> + '_ {
        self.headers.iter().map(|(n, v)| {
            let title = n
                .split('-')
                .map(|word| {
                    let mut chars: Vec<char> = word.chars().collect();
                    if let Some(first) = chars.first_mut() {
                        *first = first.to_ascii_uppercase();
                    }
                    for c in chars.iter_mut().skip(1) {
                        *c = c.to_ascii_lowercase();
                    }
                    chars.into_iter().collect::<String>()
                })
                .collect::<Vec<_>>()
                .join("-");
            (title, v.as_str())
        })
    }

    /// Get all headers as-is with original casing.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.headers.iter().map(|(n, v)| (n.as_str(), v.as_str()))
    }

    /// Number of headers.
    pub fn len(&self) -> usize {
        self.headers.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.headers.is_empty()
    }
}

/// Generate Sec-CH-UA header value for Chrome-based browsers.
///
/// Format: `"Brand";v="version", ...`
pub fn generate_sec_ch_ua(browser: &str, version: u16, include_not_a_brand: bool) -> String {
    let mut brands = Vec::new();

    // Chromium base
    brands.push(format!("\"Chromium\";v=\"{}\"", version));

    // Browser brand
    match browser.to_lowercase().as_str() {
        "chrome" => brands.push(format!("\"Google Chrome\";v=\"{}\"", version)),
        "edge" => brands.push(format!("\"Microsoft Edge\";v=\"{}\"", version)),
        "opera" => brands.push(format!("\"Opera\";v=\"{}\"", version)),
        _ => brands.push(format!("\"{}\";v=\"{}\"", browser, version)),
    }

    // "Not-A.Brand" fake brand for entropy
    if include_not_a_brand {
        brands.push("\"Not-A.Brand\";v=\"99\"".to_string());
    }

    brands.join(", ")
}

/// Generate Sec-CH-UA-Full-Version-List header.
pub fn generate_sec_ch_ua_full(browser: &str, version: &str) -> String {
    let _major = version.split('.').next().unwrap_or("100");

    format!(
        "\"Chromium\";v=\"{}\", \"{}\";v=\"{}\", \"Not-A.Brand\";v=\"99.0.0.0\"",
        version,
        match browser.to_lowercase().as_str() {
            "chrome" => "Google Chrome",
            "edge" => "Microsoft Edge",
            "opera" => "Opera",
            _ => browser,
        },
        version
    )
}

#[cfg(test)]
mod case_tests {
    use super::*;

    #[test]
    fn test_case_sensitive_insert() {
        let mut headers = CaseSensitiveHeaders::new();
        headers.insert("Content-Type", "application/json");
        headers.insert("content-type", "text/html"); // Should update

        assert_eq!(headers.get("content-type"), Some("text/html"));
        assert_eq!(headers.len(), 1);
    }

    #[test]
    fn test_title_case() {
        let mut headers = CaseSensitiveHeaders::new();
        headers.insert("user-agent", "test");
        headers.insert("accept-encoding", "gzip");

        let title_cased: Vec<_> = headers.as_title_case().collect();
        assert_eq!(title_cased[0].0, "User-Agent");
        assert_eq!(title_cased[1].0, "Accept-Encoding");
    }

    #[test]
    fn test_sec_ch_ua_chrome() {
        let ua = generate_sec_ch_ua("Chrome", 143, true);
        assert!(ua.contains("Chromium"));
        assert!(ua.contains("Google Chrome"));
        assert!(ua.contains("Not-A.Brand"));
    }

    #[test]
    fn test_sec_ch_ua_edge() {
        let ua = generate_sec_ch_ua("Edge", 142, false);
        assert!(ua.contains("Microsoft Edge"));
        assert!(!ua.contains("Not-A.Brand"));
    }
}
