//! HTTP/2 Fingerprint Emulation Types
//!
//! These types enable precise HTTP/2 fingerprint emulation to match browser behavior.
//! Anti-bot systems fingerprint HTTP/2 at multiple levels:
//! - SETTINGS frame parameter order
//! - Pseudo-header field order in HEADERS frames
//! - PRIORITY frames sent after handshake
//! - Window sizes and frame limits
//!
//! This module provides types to configure all these aspects.

use std::time::Duration;

// Re-export from http2 crate for fingerprint control
pub use http2::frame::{
    ExperimentalSettings, Priorities, PrioritiesBuilder, Priority, PseudoId, PseudoOrder, Setting,
    SettingId, SettingsOrder, SettingsOrderBuilder, StreamDependency, StreamId,
};

/// Extended HTTP/2 settings with fingerprint emulation support.
///
/// HTTP/2 settings with fingerprint emulation support.
/// fingerprint emulation including pseudo-header order, settings order,
/// and priority frames.
///
/// # Browser Differences
///
/// Different browsers send HTTP/2 frames in different orders:
/// - **Chrome**: Specific SETTINGS order, sends PRIORITY frames
/// - **Firefox**: Different pseudo-header order, larger header tables
/// - **Safari**: Smaller header tables, no priority frames
#[derive(Debug, Clone)]
pub struct H2Fingerprint {
    // Standard SETTINGS frame values
    /// SETTINGS_HEADER_TABLE_SIZE (0x1) - HPACK dynamic table size
    pub header_table_size: Option<u32>,
    /// SETTINGS_ENABLE_PUSH (0x2) - Server push enabled
    pub enable_push: Option<bool>,
    /// SETTINGS_MAX_CONCURRENT_STREAMS (0x3)
    pub max_concurrent_streams: Option<u32>,
    /// SETTINGS_INITIAL_WINDOW_SIZE (0x4) - Stream-level flow control
    pub initial_window_size: u32,
    /// Connection-level flow control window
    pub initial_conn_window_size: u32,
    /// SETTINGS_MAX_FRAME_SIZE (0x5) - Maximum frame payload
    pub max_frame_size: Option<u32>,
    /// SETTINGS_MAX_HEADER_LIST_SIZE (0x6) - Maximum header block size
    pub max_header_list_size: Option<u32>,

    // Fingerprint emulation (advanced)
    /// Order of pseudo-header fields (:method, :path, etc.) in HEADERS frame
    pub pseudo_order: Option<PseudoOrder>,
    /// Order of SETTINGS parameters in initial SETTINGS frame
    pub settings_order: Option<SettingsOrder>,
    /// PRIORITY frames to send after connection establishment
    pub priorities: Option<Priorities>,
    /// Stream dependency for outgoing HEADERS frame
    pub stream_dependency: Option<StreamDependency>,
    /// Experimental SETTINGS (for future protocols)
    pub experimental_settings: Option<ExperimentalSettings>,

    // Keep-alive
    /// Interval for HTTP/2 PING keep-alive frames
    pub keep_alive_interval: Option<Duration>,
    /// Timeout for receiving PING acknowledgement
    pub keep_alive_timeout: Option<Duration>,
    /// Whether to send keep-alive PINGs while connection is idle
    pub keep_alive_while_idle: bool,

    // Advanced options
    /// Initial stream ID
    pub initial_stream_id: Option<u32>,
    /// Whether to use adaptive flow control
    pub adaptive_window: bool,
    /// Whether to disable RFC 7540 priorities
    pub no_rfc7540_priorities: Option<bool>,
    /// Enable CONNECT protocol (RFC 8441)
    pub enable_connect_protocol: Option<bool>,
}

impl Default for H2Fingerprint {
    fn default() -> Self {
        Self::chrome()
    }
}

impl H2Fingerprint {
    /// Chrome 120+ HTTP/2 fingerprint.
    ///
    /// Based on Chromium source and real browser captures.
    /// Chrome uses large windows (6MB), specific SETTINGS order,
    /// and sends PRIORITY frames after handshake.
    pub fn chrome() -> Self {
        Self {
            header_table_size: Some(65536),
            enable_push: Some(false), // Chrome disabled push in 2022
            max_concurrent_streams: Some(1000),
            initial_window_size: 6291456, // 6MB - Chrome's aggressive window
            initial_conn_window_size: 15728640, // 15MB
            max_frame_size: Some(16384),  // 16KB - RFC default
            max_header_list_size: Some(262144), // 256KB
            pseudo_order: Some(chrome_pseudo_order()),
            settings_order: Some(chrome_settings_order()),
            priorities: Some(chrome_priorities()),
            stream_dependency: None,
            experimental_settings: None,
            keep_alive_interval: None,
            keep_alive_timeout: None,
            keep_alive_while_idle: false,
            initial_stream_id: None,
            adaptive_window: false,
            no_rfc7540_priorities: None,
            enable_connect_protocol: None,
        }
    }

    /// Firefox 120+ HTTP/2 fingerprint.
    pub fn firefox() -> Self {
        Self {
            header_table_size: Some(65536),
            enable_push: Some(false),
            max_concurrent_streams: Some(100),
            initial_window_size: 65535,         // RFC default 64KB
            initial_conn_window_size: 12582912, // 12MB
            max_frame_size: Some(16384),
            max_header_list_size: Some(65536),
            pseudo_order: Some(firefox_pseudo_order()),
            settings_order: Some(firefox_settings_order()),
            priorities: None, // Firefox doesn't send initial priorities
            stream_dependency: None,
            experimental_settings: None,
            keep_alive_interval: None,
            keep_alive_timeout: None,
            keep_alive_while_idle: false,
            initial_stream_id: None,
            adaptive_window: false,
            no_rfc7540_priorities: Some(true), // Firefox uses RFC 9218
            enable_connect_protocol: None,
        }
    }

    /// Safari 17+ HTTP/2 fingerprint.
    pub fn safari() -> Self {
        Self {
            header_table_size: Some(4096), // Safari uses smaller table
            enable_push: Some(false),
            max_concurrent_streams: Some(100),
            initial_window_size: 65535,
            initial_conn_window_size: 10485760, // 10MB
            max_frame_size: Some(16384),
            max_header_list_size: None, // Safari doesn't send this
            pseudo_order: Some(safari_pseudo_order()),
            settings_order: Some(safari_settings_order()),
            priorities: None,
            stream_dependency: None,
            experimental_settings: None,
            keep_alive_interval: None,
            keep_alive_timeout: None,
            keep_alive_while_idle: false,
            initial_stream_id: None,
            adaptive_window: false,
            no_rfc7540_priorities: None,
            enable_connect_protocol: None,
        }
    }

    /// Create a custom fingerprint with builder pattern.
    pub fn builder() -> H2FingerprintBuilder {
        H2FingerprintBuilder {
            inner: Self::default(),
        }
    }
}

/// Builder for H2Fingerprint.
#[derive(Debug)]
pub struct H2FingerprintBuilder {
    inner: H2Fingerprint,
}

impl H2FingerprintBuilder {
    pub fn header_table_size(mut self, size: u32) -> Self {
        self.inner.header_table_size = Some(size);
        self
    }

    pub fn initial_window_size(mut self, size: u32) -> Self {
        self.inner.initial_window_size = size;
        self
    }

    pub fn initial_conn_window_size(mut self, size: u32) -> Self {
        self.inner.initial_conn_window_size = size;
        self
    }

    pub fn max_concurrent_streams(mut self, max: u32) -> Self {
        self.inner.max_concurrent_streams = Some(max);
        self
    }

    pub fn max_frame_size(mut self, size: u32) -> Self {
        self.inner.max_frame_size = Some(size);
        self
    }

    pub fn max_header_list_size(mut self, size: u32) -> Self {
        self.inner.max_header_list_size = Some(size);
        self
    }

    pub fn pseudo_order(mut self, order: PseudoOrder) -> Self {
        self.inner.pseudo_order = Some(order);
        self
    }

    pub fn settings_order(mut self, order: SettingsOrder) -> Self {
        self.inner.settings_order = Some(order);
        self
    }

    pub fn priorities(mut self, priorities: Priorities) -> Self {
        self.inner.priorities = Some(priorities);
        self
    }

    pub fn keep_alive_interval(mut self, interval: Duration) -> Self {
        self.inner.keep_alive_interval = Some(interval);
        self
    }

    pub fn keep_alive_timeout(mut self, timeout: Duration) -> Self {
        self.inner.keep_alive_timeout = Some(timeout);
        self
    }

    pub fn build(self) -> H2Fingerprint {
        self.inner
    }
}

// --- Browser-specific configurations ---

fn chrome_pseudo_order() -> PseudoOrder {
    // Chrome: :method, :authority, :scheme, :path
    PseudoOrder::new([
        PseudoId::Method,
        PseudoId::Authority,
        PseudoId::Scheme,
        PseudoId::Path,
    ])
}

fn firefox_pseudo_order() -> PseudoOrder {
    // Firefox: :method, :path, :authority, :scheme
    PseudoOrder::new([
        PseudoId::Method,
        PseudoId::Path,
        PseudoId::Authority,
        PseudoId::Scheme,
    ])
}

fn safari_pseudo_order() -> PseudoOrder {
    // Safari: :method, :scheme, :path, :authority
    PseudoOrder::new([
        PseudoId::Method,
        PseudoId::Scheme,
        PseudoId::Path,
        PseudoId::Authority,
    ])
}

fn chrome_settings_order() -> SettingsOrder {
    // Chrome SETTINGS order
    SettingsOrderBuilder::new()
        .header_table_size()
        .enable_push()
        .max_concurrent_streams()
        .initial_window_size()
        .max_frame_size()
        .max_header_list_size()
        .build()
}

fn firefox_settings_order() -> SettingsOrder {
    // Firefox SETTINGS order (different from Chrome)
    SettingsOrderBuilder::new()
        .header_table_size()
        .initial_window_size()
        .max_frame_size()
        .build()
}

fn safari_settings_order() -> SettingsOrder {
    // Safari SETTINGS order
    SettingsOrderBuilder::new()
        .enable_push()
        .initial_window_size()
        .header_table_size()
        .max_concurrent_streams()
        .max_frame_size()
        .build()
}

fn chrome_priorities() -> Priorities {
    // Chrome sends these PRIORITY frames after handshake
    // This creates a priority tree for resource scheduling
    PrioritiesBuilder::new()
        .priority(Priority::new(
            StreamId::new(3),
            StreamDependency::new(0, 200, false),
        ))
        .priority(Priority::new(
            StreamId::new(5),
            StreamDependency::new(0, 100, false),
        ))
        .priority(Priority::new(
            StreamId::new(7),
            StreamDependency::new(0, 0, false),
        ))
        .priority(Priority::new(
            StreamId::new(9),
            StreamDependency::new(7, 0, false),
        ))
        .priority(Priority::new(
            StreamId::new(11),
            StreamDependency::new(3, 0, false),
        ))
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chrome_defaults() {
        let fp = H2Fingerprint::chrome();
        assert_eq!(fp.initial_window_size, 6291456);
        assert_eq!(fp.initial_conn_window_size, 15728640);
        assert!(fp.pseudo_order.is_some());
        assert!(fp.settings_order.is_some());
        assert!(fp.priorities.is_some());
    }

    #[test]
    fn test_firefox_defaults() {
        let fp = H2Fingerprint::firefox();
        assert_eq!(fp.initial_window_size, 65535);
        assert!(fp.priorities.is_none()); // Firefox doesn't send initial priorities
    }

    #[test]
    fn test_safari_defaults() {
        let fp = H2Fingerprint::safari();
        assert_eq!(fp.header_table_size, Some(4096)); // Safari uses smaller table
        assert!(fp.max_header_list_size.is_none()); // Safari doesn't send this
    }

    #[test]
    fn test_builder() {
        let fp = H2Fingerprint::builder()
            .initial_window_size(1024 * 1024)
            .max_concurrent_streams(500)
            .build();
        assert_eq!(fp.initial_window_size, 1024 * 1024);
        assert_eq!(fp.max_concurrent_streams, Some(500));
    }

    #[test]
    fn test_default_is_chrome() {
        let default = H2Fingerprint::default();
        let chrome = H2Fingerprint::chrome();
        assert_eq!(default.initial_window_size, chrome.initial_window_size);
    }
}
