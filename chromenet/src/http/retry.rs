//! Retry logic with exponential backoff.
//!
//! Based on Chromium's `HttpNetworkTransaction::RetryReason` enum and retry logic.
//! See: net/http/http_network_transaction.h

use std::time::Duration;

/// Reasons for retrying a request (mirrors Chromium's RetryReason enum).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryReason {
    /// Server closed connection unexpectedly
    ConnectionReset,
    /// Connection was closed during request
    ConnectionClosed,
    /// Connection was aborted
    ConnectionAborted,
    /// Socket not connected
    SocketNotConnected,
    /// Empty response received
    EmptyResponse,
    /// HTTP request timeout
    HttpRequestTimeout,
    /// HTTP/2 ping failed
    Http2PingFailed,
    /// HTTP/2 server refused stream
    Http2ServerRefusedStream,
    /// TLS early data rejected
    EarlyDataRejected,
}

impl RetryReason {
    /// Map a NetError to a RetryReason, if the error is retryable.
    pub fn from_error(error: &crate::base::neterror::NetError) -> Option<Self> {
        use crate::base::neterror::NetError;

        match error {
            NetError::ConnectionReset => Some(Self::ConnectionReset),
            NetError::ConnectionClosed => Some(Self::ConnectionClosed),
            NetError::ConnectionAborted => Some(Self::ConnectionAborted),
            NetError::SocketNotConnected => Some(Self::SocketNotConnected),
            NetError::EmptyResponse => Some(Self::EmptyResponse),
            NetError::ConnectionTimedOut => Some(Self::HttpRequestTimeout),
            _ => None,
        }
    }
}

/// Configuration for retry behavior.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts (default: 3, matching Chromium)
    pub max_attempts: usize,
    /// Base delay for exponential backoff in milliseconds (default: 100)
    pub base_delay_ms: u64,
    /// Maximum delay cap in milliseconds (default: 5000)
    pub max_delay_ms: u64,
    /// Jitter factor (0.0-1.0) to randomize delays (default: 0.1)
    pub jitter_factor: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay_ms: 100,
            max_delay_ms: 5000,
            jitter_factor: 0.1,
        }
    }
}

impl RetryConfig {
    /// Create a config with no retries.
    pub fn no_retry() -> Self {
        Self {
            max_attempts: 0,
            ..Default::default()
        }
    }

    /// Create a config with aggressive retries.
    pub fn aggressive() -> Self {
        Self {
            max_attempts: 5,
            base_delay_ms: 50,
            max_delay_ms: 10000,
            jitter_factor: 0.2,
        }
    }
}

/// Calculate backoff delay for a given attempt.
///
/// Uses exponential backoff: `base_delay * 2^attempt`
/// Capped at `max_delay_ms`.
pub fn calculate_backoff(attempt: usize, config: &RetryConfig) -> Duration {
    if attempt == 0 {
        return Duration::ZERO;
    }

    // Exponential: base * 2^(attempt-1)
    let delay_ms = config
        .base_delay_ms
        .saturating_mul(1 << (attempt - 1).min(10));
    let capped_ms = delay_ms.min(config.max_delay_ms);

    // Add jitter
    let jitter_range = (capped_ms as f64 * config.jitter_factor) as u64;
    let jittered_ms = if jitter_range > 0 {
        // Simple deterministic jitter based on attempt number
        let jitter = (attempt as u64 * 7) % jitter_range;
        capped_ms.saturating_add(jitter)
    } else {
        capped_ms
    };

    Duration::from_millis(jittered_ms)
}

/// Check if we should retry based on attempt count.
pub fn should_retry(attempt: usize, config: &RetryConfig) -> bool {
    attempt < config.max_attempts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backoff_exponential() {
        let config = RetryConfig {
            jitter_factor: 0.0,
            ..Default::default()
        };

        assert_eq!(calculate_backoff(0, &config), Duration::ZERO);
        assert_eq!(calculate_backoff(1, &config), Duration::from_millis(100));
        assert_eq!(calculate_backoff(2, &config), Duration::from_millis(200));
        assert_eq!(calculate_backoff(3, &config), Duration::from_millis(400));
    }

    #[test]
    fn test_backoff_capped() {
        let config = RetryConfig {
            base_delay_ms: 1000,
            max_delay_ms: 2000,
            jitter_factor: 0.0,
            ..Default::default()
        };

        assert_eq!(calculate_backoff(1, &config), Duration::from_millis(1000));
        assert_eq!(calculate_backoff(2, &config), Duration::from_millis(2000)); // capped
        assert_eq!(calculate_backoff(3, &config), Duration::from_millis(2000)); // capped
    }

    #[test]
    fn test_should_retry() {
        let config = RetryConfig::default(); // max_attempts = 3

        assert!(should_retry(0, &config));
        assert!(should_retry(1, &config));
        assert!(should_retry(2, &config));
        assert!(!should_retry(3, &config));
        assert!(!should_retry(4, &config));
    }

    #[test]
    fn test_no_retry_config() {
        let config = RetryConfig::no_retry();
        assert!(!should_retry(0, &config));
    }
}
