//! Browser emulation module.
//!
//! Provides browser fingerprint emulation combining:
//! - TLS fingerprinting (cipher suites, extensions, curves)
//! - HTTP/2 fingerprinting (settings, priorities, pseudo-order)
//! - HTTP/1.1 options
//! - Default headers (User-Agent, Accept, etc.)

mod factory;
pub mod profiles;

pub use factory::{Emulation, EmulationBuilder, EmulationFactory};

use crate::http::H2Fingerprint;

/// HTTP/1.1 protocol options.
#[derive(Debug, Clone, Default)]
pub struct Http1Options {
    /// Title case headers (HTTP/1.1 only).
    pub title_case_headers: bool,
    /// Preserve header order.
    pub preserve_header_order: bool,
}

impl Http1Options {
    /// Create new builder.
    pub fn builder() -> Http1OptionsBuilder {
        Http1OptionsBuilder::default()
    }
}

/// Builder for Http1Options.
#[derive(Debug, Clone, Default)]
pub struct Http1OptionsBuilder {
    config: Http1Options,
}

impl Http1OptionsBuilder {
    /// Set title case headers.
    pub fn title_case_headers(mut self, enabled: bool) -> Self {
        self.config.title_case_headers = enabled;
        self
    }

    /// Set preserve header order.
    pub fn preserve_header_order(mut self, enabled: bool) -> Self {
        self.config.preserve_header_order = enabled;
        self
    }

    /// Build the options.
    pub fn build(self) -> Http1Options {
        self.config
    }
}

/// HTTP/2 protocol options.
/// Wraps H2Fingerprint with additional settings.
#[derive(Debug, Clone, Default)]
pub struct Http2Options {
    /// HTTP/2 fingerprint settings.
    pub fingerprint: Option<H2Fingerprint>,
    /// Initial window size.
    pub initial_window_size: Option<u32>,
    /// Max frame size.
    pub max_frame_size: Option<u32>,
    /// Max concurrent streams.
    pub max_concurrent_streams: Option<u32>,
    /// Max header list size.
    pub max_header_list_size: Option<u32>,
    /// Header table size.
    pub header_table_size: Option<u32>,
    /// Enable push.
    pub enable_push: Option<bool>,
}

impl Http2Options {
    /// Create new builder.
    pub fn builder() -> Http2OptionsBuilder {
        Http2OptionsBuilder::default()
    }
}

/// Builder for Http2Options.
#[derive(Debug, Clone, Default)]
pub struct Http2OptionsBuilder {
    config: Http2Options,
}

impl Http2OptionsBuilder {
    /// Set HTTP/2 fingerprint.
    pub fn fingerprint(mut self, fp: H2Fingerprint) -> Self {
        self.config.fingerprint = Some(fp);
        self
    }

    /// Set initial window size.
    pub fn initial_window_size(mut self, size: u32) -> Self {
        self.config.initial_window_size = Some(size);
        self
    }

    /// Set max frame size.
    pub fn max_frame_size(mut self, size: u32) -> Self {
        self.config.max_frame_size = Some(size);
        self
    }

    /// Set max concurrent streams.
    pub fn max_concurrent_streams(mut self, count: u32) -> Self {
        self.config.max_concurrent_streams = Some(count);
        self
    }

    /// Set max header list size.
    pub fn max_header_list_size(mut self, size: u32) -> Self {
        self.config.max_header_list_size = Some(size);
        self
    }

    /// Set header table size.
    pub fn header_table_size(mut self, size: u32) -> Self {
        self.config.header_table_size = Some(size);
        self
    }

    /// Set enable push.
    pub fn enable_push(mut self, enabled: bool) -> Self {
        self.config.enable_push = Some(enabled);
        self
    }

    /// Build the options.
    pub fn build(self) -> Http2Options {
        self.config
    }
}
