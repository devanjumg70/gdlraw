//! HTTP Client with builder pattern.
//!
//! Provides a high-level, ergonomic API for making HTTP requests with
//! browser emulation support.
//!
//! # Example
//!
//! ```rust,ignore
//! use chromenet::{Client, emulation::profiles::chrome::Chrome};
//!
//! let client = Client::builder()
//!     .emulation(Chrome::V140)
//!     .build();
//!
//! let resp = client.get("https://example.com")
//!     .send()
//!     .await?;
//! ```

use crate::base::neterror::NetError;
use crate::cookies::monster::CookieMonster;
use crate::emulation::{Emulation, EmulationFactory};
use crate::http::streamfactory::HttpStreamFactory;
use crate::socket::pool::ClientSocketPool;
use crate::socket::proxy::ProxySettings;
use crate::socket::tls::TlsOptions;
use crate::urlrequest::job::URLRequestHttpJob;
use http::Method;
use std::sync::Arc;
use std::time::Duration;
use url::Url;

/// HTTP Client for making requests.
///
/// Use [`Client::builder()`] to configure and create a client.
#[derive(Clone)]
#[allow(dead_code)] // Fields reserved for future features
pub struct Client {
    pool: Arc<ClientSocketPool>,
    factory: Arc<HttpStreamFactory>,
    cookie_store: Arc<CookieMonster>,
    emulation: Option<Emulation>,
    proxy: Option<ProxySettings>,
    timeout: Option<Duration>,
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

impl Client {
    /// Create a new client with default settings.
    pub fn new() -> Self {
        Self {
            pool: Arc::new(ClientSocketPool::default()),
            factory: Arc::new(HttpStreamFactory::new(
                Arc::new(ClientSocketPool::default()),
            )),
            cookie_store: Arc::new(CookieMonster::new()),
            emulation: None,
            proxy: None,
            timeout: None,
        }
    }

    /// Create a new client builder.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    /// Start building a GET request.
    pub fn get<U: AsRef<str>>(&self, url: U) -> RequestBuilder {
        self.request(Method::GET, url)
    }

    /// Start building a POST request.
    pub fn post<U: AsRef<str>>(&self, url: U) -> RequestBuilder {
        self.request(Method::POST, url)
    }

    /// Start building a PUT request.
    pub fn put<U: AsRef<str>>(&self, url: U) -> RequestBuilder {
        self.request(Method::PUT, url)
    }

    /// Start building a DELETE request.
    pub fn delete<U: AsRef<str>>(&self, url: U) -> RequestBuilder {
        self.request(Method::DELETE, url)
    }

    /// Start building a HEAD request.
    pub fn head<U: AsRef<str>>(&self, url: U) -> RequestBuilder {
        self.request(Method::HEAD, url)
    }

    /// Start building a PATCH request.
    pub fn patch<U: AsRef<str>>(&self, url: U) -> RequestBuilder {
        self.request(Method::PATCH, url)
    }

    /// Start building a request with custom method.
    pub fn request<U: AsRef<str>>(&self, method: Method, url: U) -> RequestBuilder {
        RequestBuilder {
            client: self.clone(),
            method,
            url: url.as_ref().to_string(),
            headers: http::HeaderMap::new(),
            body: None,
            emulation_override: None,
        }
    }
}

/// Builder for creating a [`Client`].
#[derive(Default)]
#[allow(dead_code)] // Fields reserved for future features
pub struct ClientBuilder {
    emulation: Option<Emulation>,
    cookie_store: Option<CookieMonster>,
    proxy: Option<ProxySettings>,
    tls_options: Option<TlsOptions>,
    timeout: Option<Duration>,
    pool_size_per_host: Option<usize>,
}

impl ClientBuilder {
    /// Set browser emulation.
    pub fn emulation<E: EmulationFactory>(mut self, emulation: E) -> Self {
        self.emulation = Some(emulation.emulation());
        self
    }

    /// Set cookie store.
    pub fn cookie_store(mut self, store: CookieMonster) -> Self {
        self.cookie_store = Some(store);
        self
    }

    /// Set proxy.
    pub fn proxy(mut self, proxy: ProxySettings) -> Self {
        self.proxy = Some(proxy);
        self
    }

    /// Set TLS options (overrides emulation TLS if set).
    pub fn tls_options(mut self, opts: TlsOptions) -> Self {
        self.tls_options = Some(opts);
        self
    }

    /// Set request timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Build the client.
    pub fn build(self) -> Client {
        let tls_opts = self
            .tls_options
            .or_else(|| self.emulation.as_ref().and_then(|e| e.tls_options.clone()));

        let pool = Arc::new(ClientSocketPool::new(tls_opts));
        let factory = Arc::new(HttpStreamFactory::new(pool.clone()));
        let cookie_store = Arc::new(self.cookie_store.unwrap_or_default());

        Client {
            pool,
            factory,
            cookie_store,
            emulation: self.emulation,
            proxy: self.proxy,
            timeout: self.timeout,
        }
    }
}

/// Builder for a single request.
pub struct RequestBuilder {
    client: Client,
    method: Method,
    url: String,
    headers: http::HeaderMap,
    body: Option<Vec<u8>>,
    emulation_override: Option<Emulation>,
}

impl RequestBuilder {
    /// Add a header.
    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: http::header::IntoHeaderName,
        V: TryInto<http::HeaderValue>,
    {
        if let Ok(val) = value.try_into() {
            self.headers.insert(key, val);
        }
        self
    }

    /// Set request body.
    pub fn body<B: Into<Vec<u8>>>(mut self, body: B) -> Self {
        self.body = Some(body.into());
        self
    }

    /// Set JSON body.
    #[cfg(feature = "json")]
    pub fn json<T: serde::Serialize>(mut self, json: &T) -> Self {
        if let Ok(bytes) = serde_json::to_vec(json) {
            self.body = Some(bytes);
            self.headers.insert(
                http::header::CONTENT_TYPE,
                http::HeaderValue::from_static("application/json"),
            );
        }
        self
    }

    /// Override emulation for this request.
    pub fn emulation<E: EmulationFactory>(mut self, emulation: E) -> Self {
        self.emulation_override = Some(emulation.emulation());
        self
    }

    /// Send the request.
    pub async fn send(self) -> Result<crate::http::HttpResponse, NetError> {
        let url = Url::parse(&self.url).map_err(|_| NetError::InvalidUrl)?;

        // Create job using existing infrastructure
        let mut job = URLRequestHttpJob::new(
            self.client.factory.clone(),
            url,
            self.client.cookie_store.clone(),
        );

        job.set_method(self.method);

        // Apply headers from emulation
        let emulation = self
            .emulation_override
            .as_ref()
            .or(self.client.emulation.as_ref());

        if let Some(emu) = emulation {
            for (key, value) in emu.headers.iter() {
                if let Ok(k) = key.as_str().parse::<http::header::HeaderName>() {
                    if let Ok(v) = value.to_str() {
                        job.add_header(k.as_str(), v);
                    }
                }
            }

            // Apply H2 fingerprint from emulation
            if let Some(h2_opts) = &emu.http2_options {
                job.set_h2_fingerprint(h2_opts.to_h2_fingerprint());
            }
        }

        // Apply custom headers (override emulation headers)
        for (key, value) in self.headers.iter() {
            if let Ok(v) = value.to_str() {
                job.add_header(key.as_str(), v);
            }
        }

        // Apply proxy
        if let Some(ref proxy) = self.client.proxy {
            job.set_proxy(proxy.clone());
        }

        // Start the job
        job.start().await?;

        // Get response
        job.take_response().ok_or(NetError::ConnectionFailed)
    }
}
