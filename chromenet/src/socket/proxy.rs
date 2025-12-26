use url::Url;
use zeroize::Zeroizing;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProxyType {
    Http,
    Https, // Secure proxy (TLS to proxy)
    Socks5,
}

#[derive(Debug, Clone)]
pub struct ProxySettings {
    pub url: Url, // e.g. http://proxy.com:8080 or socks5://...
    pub username: Option<String>,
    pub password: Option<Zeroizing<String>>,
}

impl ProxySettings {
    pub fn new(url_str: &str) -> Option<Self> {
        let url = Url::parse(url_str).ok()?;
        // Basic validation of scheme?
        Some(Self {
            url,
            username: None,
            password: None,
        })
    }

    pub fn with_auth(mut self, user: &str, pass: &str) -> Self {
        self.username = Some(user.to_string());
        self.password = Some(Zeroizing::new(pass.to_string()));
        self
    }

    pub fn proxy_type(&self) -> ProxyType {
        match self.url.scheme() {
            "https" => ProxyType::Https,
            "socks5" => ProxyType::Socks5,
            _ => ProxyType::Http,
        }
    }

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
}
