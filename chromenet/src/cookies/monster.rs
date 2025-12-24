use crate::cookies::canonical_cookie::CanonicalCookie;
use dashmap::DashMap;
use std::sync::Arc;
use time::OffsetDateTime;
use url::Url;

/// Maximum cookies per domain (Chromium default).
const MAX_COOKIES_PER_DOMAIN: usize = 50;

/// Maximum total cookies (Chromium default is higher, we use 3000).
#[allow(dead_code)]
const MAX_COOKIES_TOTAL: usize = 3000;

/// The main entry point for cookie management.
/// Modeled after Chromium's `net::CookieMonster`.
pub struct CookieMonster {
    // Store: Map<Domain, List<Cookie>>
    // Using DashMap for high concurrency.
    store: Arc<DashMap<String, Vec<CanonicalCookie>>>,
}

impl Default for CookieMonster {
    fn default() -> Self {
        Self::new()
    }
}

impl CookieMonster {
    pub fn new() -> Self {
        Self { store: Arc::new(DashMap::new()) }
    }

    pub fn set_canonical_cookie(&self, cookie: CanonicalCookie) {
        let mut entry = self.store.entry(cookie.domain.clone()).or_default();

        // Remove existing if name/domain/path match
        entry.retain(|c| c.name != cookie.name || c.path != cookie.path);

        // Enforce per-domain limit with LRU eviction
        while entry.len() >= MAX_COOKIES_PER_DOMAIN {
            // Remove oldest cookie (by creation_time)
            if let Some(oldest_idx) =
                entry.iter().enumerate().min_by_key(|(_, c)| c.creation_time).map(|(i, _)| i)
            {
                entry.remove(oldest_idx);
            } else {
                break;
            }
        }

        entry.push(cookie);
        drop(entry); // Release lock before checking global count

        // Enforce global MAX_COOKIES_TOTAL limit
        self.enforce_global_limit();
    }

    /// Enforce the global cookie limit by evicting oldest cookies.
    fn enforce_global_limit(&self) {
        while self.total_cookie_count() > MAX_COOKIES_TOTAL {
            // Find and remove the oldest cookie across all domains
            let mut oldest: Option<(String, usize, OffsetDateTime)> = None;

            for entry in self.store.iter() {
                let domain = entry.key().clone();
                for (idx, cookie) in entry.value().iter().enumerate() {
                    let dominated = oldest
                        .as_ref()
                        .is_some_and(|(_, _, oldest_time)| cookie.creation_time < *oldest_time);
                    if oldest.is_none() || dominated {
                        oldest = Some((domain.clone(), idx, cookie.creation_time));
                    }
                }
            }

            if let Some((domain, idx, _)) = oldest {
                if let Some(mut entry) = self.store.get_mut(&domain) {
                    if idx < entry.len() {
                        entry.remove(idx);
                    }
                }
            } else {
                break;
            }
        }
    }

    /// Get cookies matching the URL with proper domain suffix matching.
    pub fn get_cookies_for_url(&self, url: &Url) -> Vec<CanonicalCookie> {
        let mut result = Vec::new();
        let host = url.host_str().unwrap_or("");
        let now = OffsetDateTime::now_utc();

        // Collect matching domains (host itself and parent domains)
        let domains_to_check = Self::get_matching_domains(host);

        for domain in domains_to_check {
            if let Some(entry) = self.store.get(&domain) {
                for cookie in entry.iter() {
                    // Check domain match
                    if !Self::domain_matches(&cookie.domain, host, cookie.host_only) {
                        continue;
                    }

                    // Check path
                    if !Self::path_matches(&cookie.path, url.path()) {
                        continue;
                    }

                    // Check secure
                    if cookie.secure && url.scheme() != "https" {
                        continue;
                    }

                    // Check expiry
                    if cookie.is_expired(now) {
                        continue;
                    }

                    result.push(cookie.clone());
                }
            }
        }

        // Sort by path length (longest first) then creation time
        result.sort_by(|a, b| {
            b.path.len().cmp(&a.path.len()).then_with(|| a.creation_time.cmp(&b.creation_time))
        });

        result
    }

    /// Check if cookie domain matches request host.
    /// Implements RFC 6265 domain matching.
    fn domain_matches(cookie_domain: &str, request_host: &str, host_only: bool) -> bool {
        if host_only {
            // Host-only cookie: exact match required
            return cookie_domain.eq_ignore_ascii_case(request_host);
        }

        // Domain cookie: suffix match
        let cookie_domain = cookie_domain.trim_start_matches('.');

        if request_host.eq_ignore_ascii_case(cookie_domain) {
            return true;
        }

        // Check if request_host ends with .cookie_domain
        if request_host.len() > cookie_domain.len() {
            let suffix = &request_host[request_host.len() - cookie_domain.len()..];
            if suffix.eq_ignore_ascii_case(cookie_domain) {
                // Check that the character before is a dot
                let char_before =
                    request_host.chars().nth(request_host.len() - cookie_domain.len() - 1);
                return char_before == Some('.');
            }
        }

        false
    }

    /// Check if request path matches cookie path.
    /// Implements RFC 6265 path matching.
    fn path_matches(cookie_path: &str, request_path: &str) -> bool {
        if request_path == cookie_path {
            return true;
        }

        if request_path.starts_with(cookie_path) {
            // Cookie path is a prefix
            if cookie_path.ends_with('/') {
                return true;
            }
            // Check that the next character in request_path is '/'
            let next_char = request_path.chars().nth(cookie_path.len());
            return next_char == Some('/');
        }

        false
    }

    /// Get all domains to check for a given host.
    /// Returns the host itself and all parent domains.
    fn get_matching_domains(host: &str) -> Vec<String> {
        let mut domains = vec![host.to_string()];

        // Add parent domains (e.g., for "foo.bar.example.com", add "bar.example.com", "example.com")
        let parts: Vec<&str> = host.split('.').collect();
        for i in 1..parts.len().saturating_sub(1) {
            let parent = parts[i..].join(".");
            domains.push(parent);
        }

        domains
    }

    pub fn parse_and_save_cookie(&self, url: &Url, cookie_line: &str) {
        use crate::cookies::canonical_cookie::{CookiePriority, SameSite};
        use cookie::Cookie;

        if let Ok(parsed) = Cookie::parse(cookie_line) {
            let now = time::OffsetDateTime::now_utc();

            // Domain logic
            let (domain, host_only) = if let Some(d) = parsed.domain() {
                // If explicit domain, it's not host-only.
                // Chromium strips leading dot.
                let d = d.trim_start_matches('.');
                (d.to_lowercase(), false)
            } else {
                // Host only
                (url.host_str().unwrap_or("").to_lowercase(), true)
            };

            // Path logic
            let path = parsed.path().unwrap_or("/").to_string();

            // Expiry logic
            let expiration_time = parsed.expires().and_then(|e| e.datetime());

            // SameSite logic
            let same_site = match parsed.same_site() {
                Some(cookie::SameSite::Lax) => SameSite::Lax,
                Some(cookie::SameSite::Strict) => SameSite::Strict,
                Some(cookie::SameSite::None) => SameSite::NoRestriction,
                None => SameSite::Unspecified,
            };

            let c = CanonicalCookie {
                name: parsed.name().to_string(),
                value: parsed.value().to_string(),
                domain,
                path,
                creation_time: now,
                expiration_time,
                last_access_time: now,
                secure: parsed.secure().unwrap_or(false),
                http_only: parsed.http_only().unwrap_or(false),
                host_only,
                same_site,
                priority: CookiePriority::Medium,
            };

            self.set_canonical_cookie(c);
        } else {
            eprintln!("Failed to parse cookie: {}", cookie_line);
        }
    }

    /// Get total cookie count.
    pub fn total_cookie_count(&self) -> usize {
        self.store.iter().map(|e| e.value().len()).sum()
    }

    /// Clear all cookies.
    pub fn clear(&self) {
        self.store.clear();
    }

    /// Iterate over all cookies (for persistence).
    pub fn iter_all_cookies(&self) -> impl Iterator<Item = CanonicalCookie> + '_ {
        self.store.iter().flat_map(|entry| entry.value().clone())
    }
}
