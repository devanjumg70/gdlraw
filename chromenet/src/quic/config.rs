//! QUIC connection configuration.

use std::time::Duration;

/// QUIC/HTTP3 configuration.
#[derive(Debug, Clone)]
pub struct QuicConfig {
    /// Maximum idle timeout
    pub idle_timeout: Duration,
    /// Initial RTT estimate
    pub initial_rtt: Duration,
    /// Maximum UDP payload size
    pub max_udp_payload_size: u16,
    /// Initial max data (connection-level flow control)
    pub initial_max_data: u64,
    /// Initial max stream data (stream-level flow control)
    pub initial_max_stream_data: u64,
    /// Initial max bidirectional streams
    pub initial_max_streams_bidi: u64,
    /// Initial max unidirectional streams
    pub initial_max_streams_uni: u64,
    /// Enable 0-RTT
    pub enable_0rtt: bool,
    /// ALPN protocols
    pub alpn_protocols: Vec<String>,
}

impl Default for QuicConfig {
    fn default() -> Self {
        Self {
            idle_timeout: Duration::from_secs(60),
            initial_rtt: Duration::from_millis(100),
            max_udp_payload_size: 1200,
            initial_max_data: 10 * 1024 * 1024,   // 10 MB
            initial_max_stream_data: 1024 * 1024, // 1 MB
            initial_max_streams_bidi: 100,
            initial_max_streams_uni: 100,
            enable_0rtt: true,
            alpn_protocols: vec!["h3".to_string()],
        }
    }
}

impl QuicConfig {
    /// Create a new config with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set idle timeout.
    pub fn idle_timeout(mut self, timeout: Duration) -> Self {
        self.idle_timeout = timeout;
        self
    }

    /// Set initial RTT.
    pub fn initial_rtt(mut self, rtt: Duration) -> Self {
        self.initial_rtt = rtt;
        self
    }

    /// Set max UDP payload size.
    pub fn max_udp_payload_size(mut self, size: u16) -> Self {
        self.max_udp_payload_size = size;
        self
    }

    /// Set initial max data.
    pub fn initial_max_data(mut self, max: u64) -> Self {
        self.initial_max_data = max;
        self
    }

    /// Set initial max stream data.
    pub fn initial_max_stream_data(mut self, max: u64) -> Self {
        self.initial_max_stream_data = max;
        self
    }

    /// Enable or disable 0-RTT.
    pub fn enable_0rtt(mut self, enable: bool) -> Self {
        self.enable_0rtt = enable;
        self
    }

    /// Set ALPN protocols.
    pub fn alpn_protocols(mut self, protocols: Vec<String>) -> Self {
        self.alpn_protocols = protocols;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = QuicConfig::default();
        assert_eq!(config.idle_timeout, Duration::from_secs(60));
        assert!(config.enable_0rtt);
        assert!(config.alpn_protocols.contains(&"h3".to_string()));
    }

    #[test]
    fn test_builder_pattern() {
        let config = QuicConfig::new()
            .idle_timeout(Duration::from_secs(30))
            .enable_0rtt(false)
            .initial_max_data(5 * 1024 * 1024);

        assert_eq!(config.idle_timeout, Duration::from_secs(30));
        assert!(!config.enable_0rtt);
        assert_eq!(config.initial_max_data, 5 * 1024 * 1024);
    }
}
