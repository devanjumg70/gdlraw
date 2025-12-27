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
            _ => ProxyType::Http,
        }
    }

    /// Check if URL should bypass this proxy.
    pub fn should_bypass(&self, target: &Url) -> bool {
        self.bypass.should_bypass_url(target)
    }

    /// Get `Proxy-Authorization` header value.
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
