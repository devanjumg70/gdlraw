//! URL Request Context - Central configuration for network requests.
//!
//! Based on Chromium's net::URLRequestContext, provides a centralized
//! configuration point for network stack components.

use crate::cookies::monster::CookieMonster;
use crate::http::streamfactory::HttpStreamFactory;
use crate::socket::pool::ClientSocketPool;
use crate::socket::proxy::ProxySettings;
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
        }
    }
}

/// Central configuration for network requests.
///
/// Mirrors Chromium's URLRequestContext, bundling together:
/// - HTTP stream factory
/// - Socket pool
/// - Cookie store
/// - Proxy settings
/// - User-Agent and other HTTP settings
pub struct URLRequestContext {
    /// HTTP stream factory for creating connections.
    stream_factory: Arc<HttpStreamFactory>,

    /// Socket pool for connection reuse.
    socket_pool: Arc<ClientSocketPool>,

    /// Cookie storage.
    cookie_store: Arc<CookieMonster>,

    /// Configuration options.
    config: URLRequestContextConfig,
}

impl URLRequestContext {
    /// Create a new URLRequestContext with default configuration.
    pub fn new() -> Self {
        Self::with_config(URLRequestContextConfig::default())
    }

    /// Create a new URLRequestContext with custom configuration.
    pub fn with_config(config: URLRequestContextConfig) -> Self {
        let socket_pool = Arc::new(ClientSocketPool::new());
        let cookie_store = Arc::new(CookieMonster::new());
        let stream_factory = Arc::new(HttpStreamFactory::new(Arc::clone(&socket_pool)));

        // Start idle socket cleanup task
        socket_pool.start_cleanup_task();

        Self { stream_factory, socket_pool, cookie_store, config }
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
