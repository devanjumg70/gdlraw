//! Emulation factory and core types.

use crate::emulation::{Http1Options, Http2Options};
use crate::socket::tls::TlsOptions;
use http::HeaderMap;

/// Factory trait for creating emulation configurations.
///
/// Allows different types (enums, structs) to provide emulation configurations.
/// Used for predefined browser profiles and custom emulation strategies.
pub trait EmulationFactory {
    /// Create an [`Emulation`] from this factory.
    fn emulation(self) -> Emulation;
}

/// Builder for [`Emulation`] configuration.
#[derive(Debug, Clone, Default)]
#[must_use]
pub struct EmulationBuilder {
    emulation: Emulation,
}

/// HTTP emulation configuration for mimicking browsers.
///
/// Combines:
/// - TLS options (fingerprinting)
/// - HTTP/1.1 options
/// - HTTP/2 options (settings, priorities)
/// - Default headers (User-Agent, Accept, etc.)
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct Emulation {
    /// TLS fingerprint options.
    pub tls_options: Option<TlsOptions>,
    /// HTTP/1.1 protocol options.
    pub http1_options: Option<Http1Options>,
    /// HTTP/2 protocol options.
    pub http2_options: Option<Http2Options>,
    /// Default headers to include in requests.
    pub headers: HeaderMap,
}

impl Emulation {
    /// Create a new builder.
    #[inline]
    pub fn builder() -> EmulationBuilder {
        EmulationBuilder::default()
    }

    /// Get TLS options reference.
    #[inline]
    pub fn tls_options(&self) -> Option<&TlsOptions> {
        self.tls_options.as_ref()
    }

    /// Get HTTP/1.1 options reference.
    #[inline]
    pub fn http1_options(&self) -> Option<&Http1Options> {
        self.http1_options.as_ref()
    }

    /// Get HTTP/2 options reference.
    #[inline]
    pub fn http2_options(&self) -> Option<&Http2Options> {
        self.http2_options.as_ref()
    }

    /// Get headers reference.
    #[inline]
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Decompose into parts.
    pub fn into_parts(
        self,
    ) -> (
        Option<TlsOptions>,
        Option<Http1Options>,
        Option<Http2Options>,
        HeaderMap,
    ) {
        (
            self.tls_options,
            self.http1_options,
            self.http2_options,
            self.headers,
        )
    }
}

impl EmulationBuilder {
    /// Set TLS options.
    #[inline]
    pub fn tls_options(mut self, opts: TlsOptions) -> Self {
        self.emulation.tls_options = Some(opts);
        self
    }

    /// Set HTTP/1.1 options.
    #[inline]
    pub fn http1_options(mut self, opts: Http1Options) -> Self {
        self.emulation.http1_options = Some(opts);
        self
    }

    /// Set HTTP/2 options.
    #[inline]
    pub fn http2_options(mut self, opts: Http2Options) -> Self {
        self.emulation.http2_options = Some(opts);
        self
    }

    /// Set default headers.
    #[inline]
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        self.emulation.headers = headers;
        self
    }

    /// Add a single header.
    #[inline]
    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: http::header::IntoHeaderName,
        V: TryInto<http::HeaderValue>,
    {
        if let Ok(val) = value.try_into() {
            self.emulation.headers.insert(key, val);
        }
        self
    }

    /// Build the Emulation.
    #[inline]
    pub fn build(self) -> Emulation {
        self.emulation
    }
}

// EmulationFactory implementations

impl EmulationFactory for Emulation {
    #[inline]
    fn emulation(self) -> Emulation {
        self
    }
}

impl EmulationFactory for TlsOptions {
    #[inline]
    fn emulation(self) -> Emulation {
        Emulation::builder().tls_options(self).build()
    }
}

impl EmulationFactory for Http1Options {
    #[inline]
    fn emulation(self) -> Emulation {
        Emulation::builder().http1_options(self).build()
    }
}

impl EmulationFactory for Http2Options {
    #[inline]
    fn emulation(self) -> Emulation {
        Emulation::builder().http2_options(self).build()
    }
}
