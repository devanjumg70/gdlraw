use crate::cookies::canonical_cookie::CanonicalCookie;
use dashmap::DashMap;
use std::sync::Arc;
use url::Url;

/// The main entry point for cookie management.
/// Modeled after Chromium's `net::CookieMonster`.
pub struct CookieMonster {
    // Simplified store: Map<Domain, List<Cookie>>
    // Chromium uses complex multigraph. We'll start simple.
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
        // Simple domain index
        let mut entry = self.store.entry(cookie.domain.clone()).or_default();

        // Remove existing if name/domain/path match
        // Note: entry.value_mut() returns &mut Vec<...>
        entry.retain(|c| c.name != cookie.name || c.path != cookie.path);

        entry.push(cookie);
    }

    pub fn get_cookies_for_url(&self, url: &Url) -> Vec<CanonicalCookie> {
        let mut result = Vec::new();
        let host = url.host_str().unwrap_or("");

        // Very naive domain matching
        if let Some(entry) = self.store.get(host) {
            for cookie in entry.iter() {
                // Check path
                if url.path().starts_with(&cookie.path) {
                    // Check secure
                    if cookie.secure && url.scheme() != "https" {
                        continue;
                    }
                    // Check expiry?
                    result.push(cookie.clone());
                }
            }
        }

        // Also check superdomains?
        // e.g. for foo.google.com, check .google.com entries.

        result
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
                (d.to_string(), false)
            } else {
                // Host only
                (url.host_str().unwrap_or("").to_string(), true)
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
                priority: CookiePriority::Medium, // Cookie crate doesn't parse Priority
            };

            self.set_canonical_cookie(c);
        } else {
            eprintln!("Failed to parse cookie: {}", cookie_line);
        }
    }
}
