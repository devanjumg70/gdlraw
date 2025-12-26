//! URL Request Context - Central configuration for network requests.
//!
//! Based on Chromium's net::URLRequestContext, provides a centralized
//! configuration point for network stack components.

use crate::cookies::monster::CookieMonster;
use crate::dns::{DnsResolverWithOverrides, HickoryResolver, Resolve};
use crate::http::streamfactory::HttpStreamFactory;
use crate::socket::pool::ClientSocketPool;
use crate::socket::proxy::ProxySettings;
use crate::socket::tls::TlsOptions;
use crate::urlrequest::device::Device;
use std::borrow::Cow;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

/// Configuration options for URLRequestContext.
#[derive(Debug, Clone)]
pub struct URLRequestContextConfig {
    /// User-Agent string to use for requests.
    pub user_agent: String,

    /// Accept-Language header value.
    pub accept_language: Option<String>,

    /// Proxy settings (None for direct connections).
    pub proxy: Option<ProxySettings>,

    /// Maximum sockets per host group.
    pub max_sockets_per_group: usize,

    /// Maximum total sockets.
    pub max_sockets_total: usize,

    /// Emulated device.
    pub device: Option<Device>,

    /// TLS options (overrides device if both set).
    pub tls_options: Option<TlsOptions>,

    /// Custom DNS resolver (None = use HickoryResolver).
    pub dns_resolver: Option<Arc<dyn Resolve>>,

    /// DNS hostname overrides (hostname -> addresses).
    pub dns_overrides: HashMap<Cow<'static, str>, Vec<SocketAddr>>,
}

impl Default for URLRequestContextConfig {
    fn default() -> Self {
        Self {
            user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
                (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
                .to_string(),
            accept_language: Some("en-US,en;q=0.9".to_string()),
            proxy: None,
            max_sockets_per_group: 6,
            max_sockets_total: 256,
            device: None,
            tls_options: None,
            dns_resolver: None,
            dns_overrides: HashMap::new(),
        }
    }
}

impl std::fmt::Debug for URLRequestContextConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("URLRequestContextConfig")
            .field("user_agent", &self.user_agent)
            .field("accept_language", &self.accept_language)
            .field("proxy", &self.proxy)
            .field("max_sockets_per_group", &self.max_sockets_per_group)
            .field("max_sockets_total", &self.max_sockets_total)
            .field("device", &self.device)
            .field("tls_options", &self.tls_options)
            .field("dns_resolver", &self.dns_resolver.is_some())
            .field("dns_overrides_count", &self.dns_overrides.len())
            .finish()
    }
}

/// Central configuration for network requests.
///
/// Mirrors Chromium's URLRequestContext, bundling together:
/// - HTTP stream factory
/// - Socket pool
/// - Cookie store
/// - DNS resolver
/// - Proxy settings
/// - User-Agent and other HTTP settings
pub struct URLRequestContext {
    /// HTTP stream factory for creating connections.
    stream_factory: Arc<HttpStreamFactory>,

    /// Socket pool for connection reuse.
    socket_pool: Arc<ClientSocketPool>,

    /// Cookie storage.
    cookie_store: Arc<CookieMonster>,

    /// DNS resolver.
    resolver: Arc<dyn Resolve>,

    /// Configuration options.
    config: URLRequestContextConfig,
}

impl URLRequestContext {
    /// Create a new URLRequestContext with default configuration.
    pub fn new() -> Self {
        Self::with_config(URLRequestContextConfig::default())
    }

    /// Create a new URLRequestContext with custom configuration.
    pub fn with_config(mut config: URLRequestContextConfig) -> Self {
        // Derive TLS options from device if not explicitly set
        if config.tls_options.is_none() {
            if let Some(device) = &config.device {
                config.tls_options = Some(device.impersonate.create_tls_options());
            }
        }

        // Setup DNS resolver with optional overrides
        let base_resolver: Arc<dyn Resolve> = config
            .dns_resolver
            .clone()
            .unwrap_or_else(|| Arc::new(HickoryResolver::new()));

        let resolver: Arc<dyn Resolve> = if config.dns_overrides.is_empty() {
            base_resolver
        } else {
            Arc::new(DnsResolverWithOverrides::new(
                base_resolver,
                config.dns_overrides.clone(),
            ))
        };

        let socket_pool = Arc::new(ClientSocketPool::new(config.tls_options.clone()));
        let cookie_store = Arc::new(CookieMonster::new());
        let stream_factory = Arc::new(HttpStreamFactory::new(Arc::clone(&socket_pool)));

        // Start idle socket cleanup task
        socket_pool.start_cleanup_task();

        Self {
            stream_factory,
            socket_pool,
            cookie_store,
            resolver,
            config,
        }
    }

    /// Get the HTTP stream factory.
    pub fn stream_factory(&self) -> &Arc<HttpStreamFactory> {
        &self.stream_factory
    }

    /// Get the socket pool.
    pub fn socket_pool(&self) -> &Arc<ClientSocketPool> {
        &self.socket_pool
    }

    /// Get the cookie store.
    pub fn cookie_store(&self) -> &Arc<CookieMonster> {
        &self.cookie_store
    }

    /// Get the DNS resolver.
    pub fn resolver(&self) -> &Arc<dyn Resolve> {
        &self.resolver
    }

    /// Get the user agent string.
    pub fn user_agent(&self) -> &str {
        &self.config.user_agent
    }

    /// Get the accept language header.
    pub fn accept_language(&self) -> Option<&str> {
        self.config.accept_language.as_deref()
    }

    /// Get proxy settings.
    pub fn proxy(&self) -> Option<&ProxySettings> {
        self.config.proxy.as_ref()
    }
}

impl Default for URLRequestContext {
    fn default() -> Self {
        Self::new()
    }
}
