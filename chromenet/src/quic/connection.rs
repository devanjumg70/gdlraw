//! QUIC connection.

use super::config::QuicConfig;
use crate::base::neterror::NetError;
use std::net::SocketAddr;
use url::Url;

/// QUIC connection (placeholder).
///
/// Note: Full implementation requires the `quinn` crate.
pub struct QuicConnection {
    url: Url,
    remote_addr: Option<SocketAddr>,
    #[allow(dead_code)]
    config: QuicConfig,
}

impl QuicConnection {
    /// Get the URL.
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Get the remote address if connected.
    pub fn remote_addr(&self) -> Option<SocketAddr> {
        self.remote_addr
    }
}

/// Builder for QUIC connections.
#[derive(Debug)]
pub struct QuicConnectionBuilder {
    url: Option<Url>,
    config: QuicConfig,
}

impl Default for QuicConnectionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl QuicConnectionBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self {
            url: None,
            config: QuicConfig::default(),
        }
    }

    /// Set the URL to connect to.
    pub fn url(mut self, url: &str) -> Result<Self, NetError> {
        let parsed = Url::parse(url).map_err(|_| NetError::InvalidUrl)?;

        // Must be HTTPS for HTTP/3
        if parsed.scheme() != "https" {
            return Err(NetError::InvalidUrl);
        }

        self.url = Some(parsed);
        Ok(self)
    }

    /// Set the QUIC configuration.
    pub fn config(mut self, config: QuicConfig) -> Self {
        self.config = config;
        self
    }

    /// Connect to the server (placeholder).
    ///
    /// Note: Full implementation requires the `quinn` crate.
    pub async fn connect(self) -> Result<QuicConnection, NetError> {
        let _url = self.url.ok_or(NetError::InvalidUrl)?;

        // Placeholder - full implementation would:
        // 1. Resolve DNS
        // 2. Create UDP socket
        // 3. Create quinn Endpoint
        // 4. Connect with TLS (boring for certificate verification)
        // 5. Return connected QuicConnection

        Err(NetError::NotImplemented)
    }

    /// Get the URL if set.
    pub fn get_url(&self) -> Option<&Url> {
        self.url.as_ref()
    }
}

/// Connect to a QUIC server (convenience function).
#[allow(dead_code)] // API placeholder for quinn integration
pub async fn connect(url: &str) -> Result<QuicConnection, NetError> {
    QuicConnectionBuilder::new().url(url)?.connect().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_new() {
        let builder = QuicConnectionBuilder::new();
        assert!(builder.url.is_none());
    }

    #[test]
    fn test_builder_url() {
        let builder = QuicConnectionBuilder::new()
            .url("https://example.com")
            .unwrap();
        assert!(builder.url.is_some());
    }

    #[test]
    fn test_builder_invalid_scheme() {
        let result = QuicConnectionBuilder::new().url("http://example.com");
        assert!(result.is_err());
    }

    #[test]
    fn test_builder_config() {
        let config = QuicConfig::new().enable_0rtt(false);
        let builder = QuicConnectionBuilder::new().config(config);
        assert!(!builder.config.enable_0rtt);
    }

    #[tokio::test]
    async fn test_connect_not_implemented() {
        let result = QuicConnectionBuilder::new()
            .url("https://example.com")
            .unwrap()
            .connect()
            .await;

        assert!(matches!(result, Err(NetError::NotImplemented)));
    }
}
