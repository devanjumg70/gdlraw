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
        Self { headers: Vec::new() }
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
            self.headers.iter().find(|(n, _)| n == target).map(|(_, v)| v)
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
