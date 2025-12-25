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
