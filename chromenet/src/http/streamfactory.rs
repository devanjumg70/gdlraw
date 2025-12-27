//! HTTP Stream Factory
//!
//! Creates HTTP/1.1 and HTTP/2 streams for network transactions.
//! Supports H2 multiplexing and browser fingerprint emulation.

use crate::base::neterror::NetError;
use crate::http::h2fingerprint::H2Fingerprint;
use crate::socket::pool::{ClientSocketPool, PoolResult};
use dashmap::DashMap;
use http::{Request, Response};
use http2::client::conn as h2_conn;
use hyper::body::Incoming;
use hyper::client::conn::http1;
use hyper_util::rt::TokioIo;
use std::sync::Arc;
use tokio::spawn;
use url::Url;

/// Type alias for H2 sender (using http2 crate's forked h2)
type H2Sender = h2_conn::SendRequest<http_body_util::Empty<bytes::Bytes>>;

/// Wraps the underlying protocol stream (H1/H2).
pub struct HttpStream {
    inner: HttpStreamInner,
    is_reused: bool,
}

enum HttpStreamInner {
    H1(http1::SendRequest<http_body_util::Empty<bytes::Bytes>>),
    H2(H2Sender),
}

impl HttpStream {
    pub fn is_h2(&self) -> bool {
        matches!(self.inner, HttpStreamInner::H2(_))
    }

    pub fn is_reused(&self) -> bool {
        self.is_reused
    }

    pub async fn send_request(
        &mut self,
        req: Request<http_body_util::Empty<bytes::Bytes>>,
    ) -> Result<Response<Incoming>, NetError> {
        match &mut self.inner {
            HttpStreamInner::H1(sender) => sender.send_request(req).await.map_err(|e| {
                tracing::debug!("H1 request error: {:?}", e);
                NetError::ConnectionClosed
            }),
            HttpStreamInner::H2(sender) => sender.send_request(req).await.map_err(|e| {
                tracing::debug!("H2 request error: {:?}", e);
                NetError::ConnectionClosed
            }),
        }
    }
}

/// HTTP/2 session cache for multiplexing.
/// Stores active H2 senders by host:port key for reuse.
struct H2SessionCache {
    sessions: DashMap<(String, u16), H2Sender>,
}

impl H2SessionCache {
    fn new() -> Self {
        Self {
            sessions: DashMap::new(),
        }
    }

    /// Get session key from URL
    fn key(url: &Url) -> Option<(String, u16)> {
        Some((url.host_str()?.to_string(), url.port_or_known_default()?))
    }

    /// Get an existing H2 sender if available and ready
    fn get(&self, url: &Url) -> Option<H2Sender> {
        let key = Self::key(url)?;
        let entry = self.sessions.get(&key)?;
        let sender = entry.value();
        // Check if sender is still usable
        if sender.is_closed() {
            drop(entry);
            self.sessions.remove(&key);
            None
        } else {
            Some(sender.clone())
        }
    }

    /// Store an H2 sender for reuse
    fn store(&self, url: &Url, sender: H2Sender) {
        if let Some(key) = Self::key(url) {
            self.sessions.insert(key, sender);
        }
    }

    /// Remove a session (on connection error)
    #[allow(dead_code)]
    fn remove(&self, url: &Url) {
        if let Some(key) = Self::key(url) {
            self.sessions.remove(&key);
        }
    }
}

/// Factory for creating HTTP streams.
///
/// Manages connection pooling, H2 multiplexing, and applies
/// browser fingerprint settings during H2 handshake.
pub struct HttpStreamFactory {
    pool: Arc<ClientSocketPool>,
    h2_cache: H2SessionCache,
}

impl HttpStreamFactory {
    pub fn new(pool: Arc<ClientSocketPool>) -> Self {
        Self {
            pool,
            h2_cache: H2SessionCache::new(),
        }
    }

    /// Create an HTTP stream for the given URL.
    ///
    /// For HTTP/2, applies the fingerprint settings during handshake
    /// including pseudo-header order, settings order, and priority frames.
    pub async fn create_stream(
        &self,
        url: &Url,
        proxy: Option<&crate::socket::proxy::ProxySettings>,
        h2_fingerprint: Option<&H2Fingerprint>,
    ) -> Result<HttpStream, NetError> {
        // 1. Check H2 session cache for multiplexing (if HTTPS/H2)
        if url.scheme() == "https" {
            if let Some(sender) = self.h2_cache.get(url) {
                // Reuse existing H2 connection (multiplexing!)
                return Ok(HttpStream {
                    inner: HttpStreamInner::H2(sender),
                    is_reused: true,
                });
            }
        }

        // 2. Get socket from pool
        let pool_result: PoolResult = self.pool.request_socket(url, proxy).await?;

        let io = TokioIo::new(pool_result.socket);

        if pool_result.is_h2 {
            // H2 Handshake with fingerprint emulation
            let fp = h2_fingerprint.cloned().unwrap_or_default();

            // Build http2 connection with fingerprint settings
            let mut builder = h2_conn::Builder::new(io::TokioExecutor::new());

            // Apply window sizes
            builder.initial_stream_window_size(fp.initial_window_size);
            builder.initial_connection_window_size(fp.initial_conn_window_size);

            // Apply frame limits
            if let Some(max_frame_size) = fp.max_frame_size {
                builder.max_frame_size(max_frame_size);
            }
            if let Some(max_concurrent) = fp.max_concurrent_streams {
                builder.max_concurrent_streams(max_concurrent);
            }
            if let Some(max_header_list_size) = fp.max_header_list_size {
                builder.max_header_list_size(max_header_list_size);
            }
            if let Some(header_table_size) = fp.header_table_size {
                builder.header_table_size(header_table_size);
            }

            // Apply fingerprint emulation options
            if let Some(ref pseudo_order) = fp.pseudo_order {
                builder.headers_pseudo_order(pseudo_order.clone());
            }
            if let Some(ref settings_order) = fp.settings_order {
                builder.settings_order(settings_order.clone());
            }
            if let Some(ref priorities) = fp.priorities {
                builder.priorities(priorities.clone());
            }
            if let Some(ref stream_dep) = fp.stream_dependency {
                builder.headers_stream_dependency(stream_dep.clone());
            }
            if let Some(ref experimental) = fp.experimental_settings {
                builder.experimental_settings(experimental.clone());
            }

            // Apply push/connect protocol settings
            if let Some(enable_push) = fp.enable_push {
                builder.enable_push(enable_push);
            }
            if let Some(enable_connect) = fp.enable_connect_protocol {
                builder.enable_connect_protocol(enable_connect);
            }
            if let Some(no_priorities) = fp.no_rfc7540_priorities {
                builder.no_rfc7540_priorities(no_priorities);
            }

            // Apply keep-alive settings
            if let Some(interval) = fp.keep_alive_interval {
                builder.keep_alive_interval(interval);
            }
            if let Some(timeout) = fp.keep_alive_timeout {
                builder.keep_alive_timeout(timeout);
            }
            builder.keep_alive_while_idle(fp.keep_alive_while_idle);

            // Apply adaptive window
            if fp.adaptive_window {
                builder.adaptive_window(true);
            }

            // Perform handshake
            let (sender, conn) = builder
                .handshake::<_, http_body_util::Empty<bytes::Bytes>>(io)
                .await
                .map_err(|e| {
                    tracing::debug!("H2 handshake failed: {:?}", e);
                    NetError::ConnectionFailed
                })?;

            // Store sender in cache for multiplexing
            self.h2_cache.store(url, sender.clone());

            // Spawn connection driver
            spawn(async move {
                if let Err(e) = conn.await {
                    tracing::debug!("H2 connection error: {:?}", e);
                }
            });

            Ok(HttpStream {
                inner: HttpStreamInner::H2(sender),
                is_reused: pool_result.is_reused,
            })
        } else {
            // H1 Handshake (Default)
            let (sender, conn) = http1::handshake(io)
                .await
                .map_err(|_| NetError::ConnectionFailed)?;

            spawn(async move {
                if let Err(e) = conn.await {
                    tracing::debug!("H1 connection error: {:?}", e);
                }
            });

            Ok(HttpStream {
                inner: HttpStreamInner::H1(sender),
                is_reused: pool_result.is_reused,
            })
        }
    }

    pub fn report_failure(&self, url: &Url) {
        self.pool.discard_socket(url);
    }
}

// Helper for H2 executor
mod io {
    use http2::rt::Executor;
    use std::future::Future;

    #[derive(Clone)]
    pub struct TokioExecutor {
        _p: (),
    }

    impl TokioExecutor {
        pub fn new() -> Self {
            Self { _p: () }
        }
    }

    impl<F> Executor<F> for TokioExecutor
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        fn execute(&self, fut: F) {
            tokio::spawn(fut);
        }
    }
}
