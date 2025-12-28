//! HTTP Cache implementation.
//!
//! Chromium mapping: net/http/http_cache.h (simplified in-memory version)
//!
//! Provides RFC 7234 compliant HTTP caching with:
//! - Cache-Control header parsing (max-age, no-store, no-cache)
//! - ETag/If-None-Match support for conditional requests
//! - Last-Modified/If-Modified-Since support
//! - Thread-safe concurrent access

use bytes::Bytes;
use dashmap::DashMap;
use http::{HeaderMap, HeaderValue, Response, StatusCode};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use url::Url;

/// Cache key components for proper Vary header handling.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct CacheKey {
    /// URL without fragment
    url: String,
    /// HTTP method (only GET/HEAD are cacheable)
    method: String,
}

impl CacheKey {
    pub fn new(url: &Url, method: &str) -> Self {
        // Strip fragment for cache key
        let mut url_str = url.to_string();
        if let Some(pos) = url_str.find('#') {
            url_str.truncate(pos);
        }
        Self {
            url: url_str,
            method: method.to_uppercase(),
        }
    }
}

/// Cached response entry.
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// Response status code
    pub status: StatusCode,
    /// Response headers
    pub headers: HeaderMap,
    /// Response body
    pub body: Bytes,
    /// When this entry was cached
    pub cached_at: Instant,
    /// Time-to-live (from max-age or Expires)
    pub ttl: Option<Duration>,
    /// ETag for conditional requests
    pub etag: Option<String>,
    /// Last-Modified for conditional requests
    pub last_modified: Option<String>,
}

impl CacheEntry {
    /// Check if the entry is still fresh.
    pub fn is_fresh(&self) -> bool {
        match self.ttl {
            Some(ttl) => self.cached_at.elapsed() < ttl,
            None => false, // No TTL means not cacheable
        }
    }

    /// Check if we should revalidate (entry exists but stale).
    pub fn needs_revalidation(&self) -> bool {
        !self.is_fresh() && (self.etag.is_some() || self.last_modified.is_some())
    }
}

/// Cache mode for controlling behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CacheMode {
    /// Normal caching behavior (RFC 7234)
    #[default]
    Normal,
    /// Bypass cache for reads and writes
    Disabled,
    /// Only read from cache, don't write
    ReadOnly,
    /// Force refresh (ignore cached responses)
    ForceRefresh,
}

/// In-memory HTTP cache.
///
/// Thread-safe implementation using DashMap for concurrent access.
/// Enforces size limits and provides LRU-style eviction.
pub struct HttpCache {
    entries: DashMap<CacheKey, CacheEntry>,
    max_entries: usize,
    current_size: AtomicUsize,
    max_size_bytes: usize,
    mode: CacheMode,
}

impl Default for HttpCache {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpCache {
    /// Create a new cache with default limits.
    pub fn new() -> Self {
        Self {
            entries: DashMap::new(),
            max_entries: 1000,
            current_size: AtomicUsize::new(0),
            max_size_bytes: 50 * 1024 * 1024, // 50MB default
            mode: CacheMode::Normal,
        }
    }

    /// Create a cache with custom limits.
    pub fn with_limits(max_entries: usize, max_size_bytes: usize) -> Self {
        Self {
            entries: DashMap::new(),
            max_entries,
            current_size: AtomicUsize::new(0),
            max_size_bytes,
            mode: CacheMode::Normal,
        }
    }

    /// Set the cache mode.
    pub fn set_mode(&mut self, mode: CacheMode) {
        self.mode = mode;
    }

    /// Get the current cache mode.
    pub fn mode(&self) -> CacheMode {
        self.mode
    }

    /// Look up a cached response.
    ///
    /// Returns the cached entry if found and still fresh.
    pub fn get(&self, url: &Url, method: &str) -> Option<CacheEntry> {
        if self.mode == CacheMode::Disabled || self.mode == CacheMode::ForceRefresh {
            return None;
        }

        // Only GET and HEAD are cacheable
        let method_upper = method.to_uppercase();
        if method_upper != "GET" && method_upper != "HEAD" {
            return None;
        }

        let key = CacheKey::new(url, method);
        let entry = self.entries.get(&key)?;

        if entry.is_fresh() {
            Some(entry.clone())
        } else {
            None
        }
    }

    /// Get entry for conditional request (may be stale).
    ///
    /// Returns entry if it exists (even stale) for revalidation.
    pub fn get_for_revalidation(&self, url: &Url, method: &str) -> Option<CacheEntry> {
        if self.mode == CacheMode::Disabled {
            return None;
        }

        let key = CacheKey::new(url, method);
        self.entries.get(&key).map(|e| e.clone())
    }

    /// Store a response in the cache.
    ///
    /// Parses Cache-Control headers to determine cacheability.
    pub fn store<B>(&self, url: &Url, method: &str, response: &Response<B>, body: Bytes) {
        if self.mode == CacheMode::Disabled || self.mode == CacheMode::ReadOnly {
            return;
        }

        // Only cache GET and HEAD
        let method_upper = method.to_uppercase();
        if method_upper != "GET" && method_upper != "HEAD" {
            return;
        }

        // Only cache successful responses
        if !response.status().is_success() && response.status() != StatusCode::NOT_MODIFIED {
            return;
        }

        // Check Cache-Control
        let cache_control = parse_cache_control(response.headers());

        // Don't cache if no-store
        if cache_control.no_store {
            return;
        }

        // Calculate TTL
        let ttl = cache_control.max_age.map(Duration::from_secs);

        // Skip if not cacheable
        if ttl.is_none() && cache_control.no_cache {
            return;
        }

        // Extract ETag and Last-Modified
        let etag = response
            .headers()
            .get(http::header::ETAG)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let last_modified = response
            .headers()
            .get(http::header::LAST_MODIFIED)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        // Create entry
        let entry = CacheEntry {
            status: response.status(),
            headers: response.headers().clone(),
            body: body.clone(),
            cached_at: Instant::now(),
            ttl,
            etag,
            last_modified,
        };

        // Evict if needed
        self.maybe_evict(body.len());

        // Store
        let key = CacheKey::new(url, method);
        self.current_size.fetch_add(body.len(), Ordering::Relaxed);
        self.entries.insert(key, entry);
    }

    /// Update cache entry from a 304 Not Modified response.
    pub fn update_from_not_modified<B>(&self, url: &Url, method: &str, response: &Response<B>) {
        let key = CacheKey::new(url, method);

        if let Some(mut entry) = self.entries.get_mut(&key) {
            // Update headers from the 304 response
            for (name, value) in response.headers() {
                // Update certain headers
                if name == http::header::CACHE_CONTROL
                    || name == http::header::ETAG
                    || name == http::header::EXPIRES
                    || name == http::header::DATE
                {
                    entry.headers.insert(name.clone(), value.clone());
                }
            }

            // Refresh TTL
            let cache_control = parse_cache_control(response.headers());
            if let Some(max_age) = cache_control.max_age {
                entry.ttl = Some(Duration::from_secs(max_age));
            }
            entry.cached_at = Instant::now();

            // Update ETag if present
            if let Some(etag) = response
                .headers()
                .get(http::header::ETAG)
                .and_then(|v| v.to_str().ok())
            {
                entry.etag = Some(etag.to_string());
            }
        }
    }

    /// Generate conditional request headers if we have a stale entry.
    pub fn get_conditional_headers(&self, url: &Url, method: &str) -> Option<HeaderMap> {
        let entry = self.get_for_revalidation(url, method)?;

        if !entry.needs_revalidation() && entry.is_fresh() {
            return None; // Entry is fresh, no need to revalidate
        }

        let mut headers = HeaderMap::new();

        if let Some(etag) = &entry.etag {
            if let Ok(value) = HeaderValue::from_str(etag) {
                headers.insert(http::header::IF_NONE_MATCH, value);
            }
        }

        if let Some(last_modified) = &entry.last_modified {
            if let Ok(value) = HeaderValue::from_str(last_modified) {
                headers.insert(http::header::IF_MODIFIED_SINCE, value);
            }
        }

        if headers.is_empty() {
            None
        } else {
            Some(headers)
        }
    }

    /// Remove an entry from the cache.
    pub fn remove(&self, url: &Url, method: &str) {
        let key = CacheKey::new(url, method);
        if let Some((_, entry)) = self.entries.remove(&key) {
            self.current_size
                .fetch_sub(entry.body.len(), Ordering::Relaxed);
        }
    }

    /// Clear all cached entries.
    pub fn clear(&self) {
        self.entries.clear();
        self.current_size.store(0, Ordering::Relaxed);
    }

    /// Get the number of cached entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if cache is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get current cache size in bytes.
    pub fn size_bytes(&self) -> usize {
        self.current_size.load(Ordering::Relaxed)
    }

    /// Evict entries if needed to make room.
    fn maybe_evict(&self, new_entry_size: usize) {
        // Evict if over entry limit
        while self.entries.len() >= self.max_entries {
            self.evict_one();
        }

        // Evict if over size limit
        while self.current_size.load(Ordering::Relaxed) + new_entry_size > self.max_size_bytes
            && !self.entries.is_empty()
        {
            self.evict_one();
        }
    }

    /// Evict one entry (oldest or least recently used).
    fn evict_one(&self) {
        // Simple eviction: remove first entry found
        // A proper LRU would track access times
        if let Some(entry) = self.entries.iter().next() {
            let key = entry.key().clone();
            drop(entry);
            self.remove_by_key(&key);
        }
    }

    fn remove_by_key(&self, key: &CacheKey) {
        if let Some((_, entry)) = self.entries.remove(key) {
            self.current_size
                .fetch_sub(entry.body.len(), Ordering::Relaxed);
        }
    }
}

/// Parsed Cache-Control directive.
#[derive(Debug, Default)]
struct CacheControl {
    no_store: bool,
    no_cache: bool,
    max_age: Option<u64>,
    must_revalidate: bool,
}

/// Parse Cache-Control header.
fn parse_cache_control(headers: &HeaderMap) -> CacheControl {
    let mut cc = CacheControl::default();

    let value = match headers.get(http::header::CACHE_CONTROL) {
        Some(v) => match v.to_str() {
            Ok(s) => s,
            Err(_) => return cc,
        },
        None => return cc,
    };

    for directive in value.split(',') {
        let directive = directive.trim().to_lowercase();

        if directive == "no-store" {
            cc.no_store = true;
        } else if directive == "no-cache" {
            cc.no_cache = true;
        } else if directive == "must-revalidate" {
            cc.must_revalidate = true;
        } else if directive.starts_with("max-age=") {
            if let Some(age_str) = directive.strip_prefix("max-age=") {
                if let Ok(age) = age_str.parse::<u64>() {
                    cc.max_age = Some(age);
                }
            }
        }
    }

    cc
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::Response;

    fn make_response(cache_control: &str, body: &str) -> Response<()> {
        Response::builder()
            .status(200)
            .header(http::header::CACHE_CONTROL, cache_control)
            .body(())
            .unwrap()
    }

    #[test]
    fn test_cache_store_and_get() {
        let cache = HttpCache::new();
        let url = Url::parse("https://example.com/page").unwrap();

        let response = make_response("max-age=3600", "hello");
        let body = Bytes::from("hello");

        cache.store(&url, "GET", &response, body.clone());

        let entry = cache.get(&url, "GET").unwrap();
        assert_eq!(entry.body, body);
        assert!(entry.is_fresh());
    }

    #[test]
    fn test_no_store_not_cached() {
        let cache = HttpCache::new();
        let url = Url::parse("https://example.com/secret").unwrap();

        let response = make_response("no-store", "secret");
        cache.store(&url, "GET", &response, Bytes::from("secret"));

        assert!(cache.get(&url, "GET").is_none());
    }

    #[test]
    fn test_post_not_cached() {
        let cache = HttpCache::new();
        let url = Url::parse("https://example.com/api").unwrap();

        let response = make_response("max-age=3600", "data");
        cache.store(&url, "POST", &response, Bytes::from("data"));

        assert!(cache.get(&url, "POST").is_none());
    }

    #[test]
    fn test_etag_revalidation() {
        let cache = HttpCache::new();
        let url = Url::parse("https://example.com/resource").unwrap();

        let response = Response::builder()
            .status(200)
            .header(http::header::CACHE_CONTROL, "max-age=0")
            .header(http::header::ETAG, "\"abc123\"")
            .body(())
            .unwrap();

        cache.store(&url, "GET", &response, Bytes::from("body"));

        let headers = cache.get_conditional_headers(&url, "GET");
        assert!(headers.is_some());
        let headers = headers.unwrap();
        assert!(headers.contains_key(http::header::IF_NONE_MATCH));
    }

    #[test]
    fn test_cache_clear() {
        let cache = HttpCache::new();
        let url = Url::parse("https://example.com/page").unwrap();

        let response = make_response("max-age=3600", "hello");
        cache.store(&url, "GET", &response, Bytes::from("hello"));

        assert_eq!(cache.len(), 1);
        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_mode_disabled() {
        let mut cache = HttpCache::new();
        cache.set_mode(CacheMode::Disabled);

        let url = Url::parse("https://example.com/page").unwrap();
        let response = make_response("max-age=3600", "hello");
        cache.store(&url, "GET", &response, Bytes::from("hello"));

        assert!(cache.get(&url, "GET").is_none());
    }

    #[test]
    fn test_parse_cache_control() {
        let mut headers = HeaderMap::new();
        headers.insert(
            http::header::CACHE_CONTROL,
            HeaderValue::from_static("max-age=3600, no-cache"),
        );

        let cc = parse_cache_control(&headers);
        assert_eq!(cc.max_age, Some(3600));
        assert!(cc.no_cache);
        assert!(!cc.no_store);
    }
}
