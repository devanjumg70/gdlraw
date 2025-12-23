use time::OffsetDateTime;

/// Represents a cookie.
/// Modeled after Chromium's `net::CanonicalCookie`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalCookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub creation_time: OffsetDateTime,
    pub expiration_time: Option<OffsetDateTime>,
    pub last_access_time: OffsetDateTime,
    pub secure: bool,
    pub http_only: bool,
    pub host_only: bool,
    pub same_site: SameSite,
    pub priority: CookiePriority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SameSite {
    Unspecified,
    NoRestriction,
    Lax,
    Strict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CookiePriority {
    Low,
    Medium,
    High,
}

impl CanonicalCookie {
    // Basic constructor for now, will expand with parsing logic later
    pub fn new(
        name: String,
        value: String,
        domain: String,
        path: String,
        creation_time: OffsetDateTime,
        expiration_time: Option<OffsetDateTime>,
    ) -> Self {
        Self {
            name,
            value,
            domain,
            path,
            creation_time,
            expiration_time,
            last_access_time: creation_time,
            secure: false,
            http_only: false,
            host_only: true, // Default to host-only if not specified
            same_site: SameSite::Unspecified,
            priority: CookiePriority::Medium,
        }
    }

    pub fn is_expired(&self, current_time: OffsetDateTime) -> bool {
        if let Some(expiry) = self.expiration_time {
            expiry < current_time
        } else {
            false // Session cookie? Or logic decided by store?
        }
    }
}
