use crate::cookies::canonicalcookie::CanonicalCookie;
use dashmap::DashMap;
use std::sync::Arc;
use time::OffsetDateTime;
use url::Url;

/// Maximum cookies per domain (Chromium default).
const MAX_COOKIES_PER_DOMAIN: usize = 50;

/// Maximum total cookies (Current Chromenet limit: 3000).
/// Chromium uses 3300, but we use a slightly lower limit to keep memory usage predictable.
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
        Self {
            store: Arc::new(DashMap::new()),
        }
    }

    pub fn set_canonical_cookie(&self, cookie: CanonicalCookie) {
        let mut entry = self.store.entry(cookie.domain.clone()).or_default();

        // Remove existing if name/domain/path match
        entry.retain(|c| c.name != cookie.name || c.path != cookie.path);

        // Enforce per-domain limit with LRU eviction
        while entry.len() >= MAX_COOKIES_PER_DOMAIN {
            // Remove oldest cookie (by creation_time)
            if let Some(oldest_idx) = entry
                .iter()
                .enumerate()
                .min_by_key(|(_, c)| c.creation_time)
                .map(|(i, _)| i)
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
            b.path
                .len()
                .cmp(&a.path.len())
                .then_with(|| a.creation_time.cmp(&b.creation_time))
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
                let char_before = request_host
                    .chars()
                    .nth(request_host.len() - cookie_domain.len() - 1);
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
        use crate::cookies::canonicalcookie::{CookiePriority, SameSite};
        use cookie::Cookie;

        if let Ok(parsed) = Cookie::parse(cookie_line) {
            let now = time::OffsetDateTime::now_utc();

            // Domain logic
            let (domain, host_only) = if let Some(d) = parsed.domain() {
                // If explicit domain, it's not host-only.
                // Chromium strips leading dot.
                let d = d.trim_start_matches('.').to_lowercase();

                // PSL validation: reject cookies set on public suffixes
                // This prevents supercookie attacks (e.g., setting cookie on ".com")
                if !crate::cookies::psl::is_valid_cookie_domain(&d, url.host_str().unwrap_or("")) {
                    return; // Silently reject like browsers do
                }

                (d, false)
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

    /// Import cookies from a browser database.
    ///
    /// This reads cookies from the specified browser and adds them to the jar.
    ///
    /// # Example
    /// ```rust,no_run
    /// use chromenet::cookies::monster::CookieMonster;
    /// use chromenet::cookies::browser::Browser;
    ///
    /// let jar = CookieMonster::new();
    /// match jar.import_from_browser(Browser::Chrome, None) {
    ///     Ok(count) => println!("Imported {} cookies", count),
    ///     Err(e) => eprintln!("Import failed: {:?}", e),
    /// }
    /// ```
    pub fn import_from_browser(
        &self,
        browser: crate::cookies::browser::Browser,
        domain_filter: Option<&str>,
    ) -> Result<usize, crate::cookies::error::CookieExtractionError> {
        use crate::cookies::browser::BrowserCookieReader;

        let mut reader = BrowserCookieReader::new(browser);
        if let Some(domain) = domain_filter {
            reader = reader.domain(domain);
        }

        let cookies = reader.read_cookies_v2()?;
        let count = cookies.len();

        for cookie in cookies {
            self.set_canonical_cookie(cookie);
        }

        Ok(count)
    }

    /// Import cookies from browser with a specific profile.
    pub fn import_from_browser_profile(
        &self,
        browser: crate::cookies::browser::Browser,
        profile: &str,
        domain_filter: Option<&str>,
    ) -> Result<usize, crate::cookies::error::CookieExtractionError> {
        use crate::cookies::browser::BrowserCookieReader;

        let mut reader = BrowserCookieReader::new(browser).with_profile(profile);
        if let Some(domain) = domain_filter {
            reader = reader.domain(domain);
        }

        let cookies = reader.read_cookies_v2()?;
        let count = cookies.len();

        for cookie in cookies {
            self.set_canonical_cookie(cookie);
        }

        Ok(count)
    }

    /// Export cookies to Netscape cookie format.
    ///
    /// The Netscape format is widely used by curl, wget, and other tools.
    /// Each line has the format:
    /// `domain\tinclude_subdomains\tpath\tsecure\texpiry\tname\tvalue`
    ///
    /// # Example
    /// ```rust,no_run
    /// use chromenet::cookies::monster::CookieMonster;
    ///
    /// let jar = CookieMonster::new();
    /// // ... add cookies ...
    /// let netscape = jar.export_netscape(None);
    /// std::fs::write("cookies.txt", netscape).unwrap();
    /// ```
    pub fn export_netscape(&self, domain_filter: Option<&str>) -> String {
        let mut lines = vec![
            "# Netscape HTTP Cookie File".to_string(),
            "# https://curl.se/docs/http-cookies.html".to_string(),
            "# This file was generated by chromenet".to_string(),
            String::new(),
        ];

        for cookie in self.iter_all_cookies() {
            // Apply domain filter if provided
            if let Some(filter) = domain_filter {
                if !cookie.domain.contains(filter) && !filter.contains(&cookie.domain) {
                    continue;
                }
            }

            // Format: domain \t include_subdomains \t path \t secure \t expiry \t name \t value
            let include_subdomains = if cookie.host_only { "FALSE" } else { "TRUE" };
            let secure = if cookie.secure { "TRUE" } else { "FALSE" };
            let expiry = cookie
                .expiration_time
                .map(|t| t.unix_timestamp())
                .unwrap_or(0);

            // Domain should start with . for non-host-only cookies
            let domain = if !cookie.host_only && !cookie.domain.starts_with('.') {
                format!(".{}", cookie.domain)
            } else {
                cookie.domain.clone()
            };

            lines.push(format!(
                "{}\t{}\t{}\t{}\t{}\t{}\t{}",
                domain, include_subdomains, cookie.path, secure, expiry, cookie.name, cookie.value
            ));
        }

        lines.join("\n")
    }

    /// Import cookies from Netscape format file content.
    ///
    /// # Example
    /// ```rust,no_run
    /// use chromenet::cookies::monster::CookieMonster;
    ///
    /// let jar = CookieMonster::new();
    /// let content = std::fs::read_to_string("cookies.txt").unwrap();
    /// let count = jar.import_netscape(&content);
    /// println!("Imported {} cookies", count);
    /// ```
    pub fn import_netscape(&self, content: &str) -> usize {
        use crate::cookies::canonicalcookie::{CookiePriority, SameSite};
        use time::OffsetDateTime;

        let mut count = 0;
        let now = OffsetDateTime::now_utc();

        for line in content.lines() {
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() < 7 {
                continue;
            }

            let domain = parts[0].to_string();
            let host_only = parts[1].eq_ignore_ascii_case("FALSE");
            let path = parts[2].to_string();
            let secure = parts[3].eq_ignore_ascii_case("TRUE");
            let expiry: i64 = parts[4].parse().unwrap_or(0);
            let name = parts[5].to_string();
            let value = parts[6].to_string();

            let expiration_time = if expiry > 0 {
                OffsetDateTime::from_unix_timestamp(expiry).ok()
            } else {
                None
            };

            let cookie = CanonicalCookie {
                name,
                value,
                domain: domain.trim_start_matches('.').to_string(),
                path,
                creation_time: now,
                expiration_time,
                last_access_time: now,
                secure,
                http_only: false, // Netscape format doesn't include httpOnly
                host_only,
                same_site: SameSite::Lax,
                priority: CookiePriority::Medium,
            };

            self.set_canonical_cookie(cookie);
            count += 1;
        }

        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cookies::canonicalcookie::{CookiePriority, SameSite};

    fn make_test_cookie(name: &str, domain: &str) -> CanonicalCookie {
        CanonicalCookie {
            name: name.to_string(),
            value: "test_value".to_string(),
            domain: domain.to_string(),
            path: "/".to_string(),
            creation_time: OffsetDateTime::now_utc(),
            expiration_time: Some(OffsetDateTime::now_utc() + time::Duration::days(30)),
            last_access_time: OffsetDateTime::now_utc(),
            secure: true,
            http_only: false,
            host_only: false,
            same_site: SameSite::Lax,
            priority: CookiePriority::Medium,
        }
    }

    #[test]
    fn test_export_netscape_basic() {
        let jar = CookieMonster::new();
        jar.set_canonical_cookie(make_test_cookie("session", "example.com"));

        let netscape = jar.export_netscape(None);
        assert!(netscape.contains("# Netscape HTTP Cookie File"));
        assert!(netscape.contains("example.com"));
        assert!(netscape.contains("session"));
    }

    #[test]
    fn test_import_netscape_basic() {
        let content = r#"# Netscape HTTP Cookie File
.example.com	TRUE	/	TRUE	1735689600	session	abc123
.test.com	FALSE	/path	FALSE	0	user	john
"#;

        let jar = CookieMonster::new();
        let count = jar.import_netscape(content);

        assert_eq!(count, 2);
        assert_eq!(jar.total_cookie_count(), 2);
    }

    #[test]
    fn test_import_export_roundtrip() {
        let jar1 = CookieMonster::new();
        jar1.set_canonical_cookie(make_test_cookie("cookie1", "example.com"));
        jar1.set_canonical_cookie(make_test_cookie("cookie2", "test.org"));

        let exported = jar1.export_netscape(None);

        let jar2 = CookieMonster::new();
        let count = jar2.import_netscape(&exported);

        assert_eq!(count, 2);
        assert_eq!(jar2.total_cookie_count(), 2);
    }

    #[test]
    fn test_export_netscape_with_filter() {
        let jar = CookieMonster::new();
        jar.set_canonical_cookie(make_test_cookie("a", "example.com"));
        jar.set_canonical_cookie(make_test_cookie("b", "other.com"));

        let filtered = jar.export_netscape(Some("example"));

        assert!(filtered.contains("example.com"));
        assert!(!filtered.contains("other.com"));
    }

    #[test]
    fn test_import_netscape_skips_comments() {
        let content = r#"# This is a comment
# Another comment

.example.com	TRUE	/	TRUE	0	test	value
# More comments
"#;

        let jar = CookieMonster::new();
        let count = jar.import_netscape(content);

        assert_eq!(count, 1);
    }
}
