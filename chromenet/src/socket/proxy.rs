use super::matcher::ProxyMatcher;
use url::Url;
use zeroize::Zeroizing;

/// Proxy protocol type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProxyType {
    /// HTTP proxy (CONNECT for HTTPS)
    Http,
    /// HTTPS proxy (TLS to proxy)
    Https,
    /// SOCKS5 proxy
    Socks5,
}

/// Proxy configuration with bypass rules.
#[derive(Debug, Clone)]
pub struct ProxySettings {
    /// Proxy URL (e.g., `http://proxy.com:8080`)
    pub url: Url,
    /// Proxy username for authentication
    pub username: Option<String>,
    /// Proxy password (zeroized on drop)
    pub password: Option<Zeroizing<String>>,
    /// NO_PROXY bypass matcher
    bypass: ProxyMatcher,
}

impl ProxySettings {
    /// Create proxy settings from URL string.
    pub fn new(url_str: &str) -> Option<Self> {
        let url = Url::parse(url_str).ok()?;
        Some(Self {
            url,
            username: None,
            password: None,
            bypass: ProxyMatcher::default(),
        })
    }

    /// Create proxy from environment variables.
    ///
    /// Checks `HTTP_PROXY`/`http_proxy` and `HTTPS_PROXY`/`https_proxy`.
    pub fn from_env() -> Option<Self> {
        let url_str = std::env::var("HTTPS_PROXY")
            .or_else(|_| std::env::var("https_proxy"))
            .or_else(|_| std::env::var("HTTP_PROXY"))
            .or_else(|_| std::env::var("http_proxy"))
            .ok()?;

        let mut settings = Self::new(&url_str)?;
        settings.bypass = ProxyMatcher::from_env();
        Some(settings)
    }

    /// Add authentication credentials.
    pub fn with_auth(mut self, user: &str, pass: &str) -> Self {
        self.username = Some(user.to_string());
        self.password = Some(Zeroizing::new(pass.to_string()));
        self
    }

    /// Add bypass rules from NO_PROXY string.
    pub fn with_bypass(mut self, no_proxy: &str) -> Self {
        self.bypass = ProxyMatcher::from_string(no_proxy);
        self
    }

    /// Get proxy type from URL scheme.
    pub fn proxy_type(&self) -> ProxyType {
        match self.url.scheme() {
            "https" => ProxyType::Https,
            "socks5" | "socks5h" => ProxyType::Socks5,
            "socks4" | "socks4a" => ProxyType::Socks5, // Treat as SOCKS
            _ => ProxyType::Http,
        }
    }

    /// Check if URL should bypass this proxy.
    pub fn should_bypass(&self, target: &Url) -> bool {
        self.bypass.should_bypass_url(target)
    }

    /// Get `Proxy-Authorization` header value for HTTP proxies.
    pub fn get_auth_header(&self) -> Option<String> {
        if let (Some(u), Some(p)) = (&self.username, &self.password) {
            use base64::{engine::general_purpose, Engine as _};
            let creds = format!("{}:{}", u, p.as_str());
            let encoded = general_purpose::STANDARD.encode(creds);
            Some(format!("Basic {}", encoded))
        } else {
            None
        }
    }

    /// Get SOCKS5 username/password for authentication.
    ///
    /// Returns (username, password) tuple for SOCKS5 auth.
    pub fn get_socks5_auth(&self) -> Option<(&str, &str)> {
        match (&self.username, &self.password) {
            (Some(u), Some(p)) => Some((u.as_str(), p.as_str())),
            _ => None,
        }
    }

    /// Check if this proxy requires authentication.
    pub fn requires_auth(&self) -> bool {
        self.username.is_some() && self.password.is_some()
    }

    /// Check if this is a SOCKS proxy.
    pub fn is_socks(&self) -> bool {
        matches!(self.proxy_type(), ProxyType::Socks5)
    }

    /// Get proxy host and port.
    pub fn host_port(&self) -> Option<(&str, u16)> {
        let host = self.url.host_str()?;
        let port = self.url.port().unwrap_or(match self.proxy_type() {
            ProxyType::Http => 80,
            ProxyType::Https => 443,
            ProxyType::Socks5 => 1080,
        });
        Some((host, port))
    }
}

/// Builder for ProxySettings.
#[derive(Default)]
pub struct ProxyBuilder {
    url: Option<Url>,
    username: Option<String>,
    password: Option<String>,
    no_proxy: String,
}

impl ProxyBuilder {
    /// Create new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set proxy URL.
    pub fn url(mut self, url: &str) -> Self {
        self.url = Url::parse(url).ok();
        self
    }

    /// Set HTTP proxy.
    pub fn http(self, url: &str) -> Self {
        self.url(&format!("http://{}", url.trim_start_matches("http://")))
    }

    /// Set HTTPS proxy.
    pub fn https(self, url: &str) -> Self {
        self.url(&format!("https://{}", url.trim_start_matches("https://")))
    }

    /// Set SOCKS5 proxy.
    pub fn socks5(self, url: &str) -> Self {
        self.url(&format!("socks5://{}", url.trim_start_matches("socks5://")))
    }

    /// Set authentication.
    pub fn auth(mut self, username: &str, password: &str) -> Self {
        self.username = Some(username.to_string());
        self.password = Some(password.to_string());
        self
    }

    /// Set NO_PROXY bypass rules.
    pub fn no_proxy(mut self, rules: &str) -> Self {
        self.no_proxy = rules.to_string();
        self
    }

    /// Build ProxySettings.
    pub fn build(self) -> Option<ProxySettings> {
        let url = self.url?;
        let bypass = if self.no_proxy.is_empty() {
            ProxyMatcher::from_env()
        } else {
            ProxyMatcher::from_string(&self.no_proxy)
        };

        Some(ProxySettings {
            url,
            username: self.username,
            password: self.password.map(Zeroizing::new),
            bypass,
        })
    }
}

/// Proxy rotation pool.
///
/// Selects proxies using round-robin or random selection.
pub struct ProxyPool {
    proxies: Vec<ProxySettings>,
    index: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    strategy: RotationStrategy,
}

/// Proxy rotation strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotationStrategy {
    /// Round-robin selection.
    RoundRobin,
    /// Random selection.
    Random,
}

impl Default for RotationStrategy {
    fn default() -> Self {
        RotationStrategy::RoundRobin
    }
}

impl ProxyPool {
    /// Create a new proxy pool with round-robin rotation.
    pub fn new(proxies: Vec<ProxySettings>) -> Self {
        Self {
            proxies,
            index: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            strategy: RotationStrategy::RoundRobin,
        }
    }

    /// Create with specific rotation strategy.
    pub fn with_strategy(mut self, strategy: RotationStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Get next proxy using the configured rotation strategy.
    pub fn next(&self) -> Option<&ProxySettings> {
        if self.proxies.is_empty() {
            return None;
        }

        match self.strategy {
            RotationStrategy::RoundRobin => {
                let idx = self.index.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Some(&self.proxies[idx % self.proxies.len()])
            }
            RotationStrategy::Random => {
                use std::time::{SystemTime, UNIX_EPOCH};
                let seed = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as usize;
                Some(&self.proxies[seed % self.proxies.len()])
            }
        }
    }

    /// Get proxy for a specific target URL (respects bypass rules).
    pub fn get_for(&self, target: &Url) -> Option<&ProxySettings> {
        self.next().filter(|p| !p.should_bypass(target))
    }

    /// Number of proxies in the pool.
    pub fn len(&self) -> usize {
        self.proxies.len()
    }

    /// Check if pool is empty.
    pub fn is_empty(&self) -> bool {
        self.proxies.is_empty()
    }
}

impl std::fmt::Debug for ProxyPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProxyPool")
            .field("count", &self.proxies.len())
            .field("strategy", &self.strategy)
            .finish()
    }
}
