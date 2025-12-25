use crate::base::neterror::NetError;
use crate::socket::pool::{ClientSocketPool, PoolResult};
use dashmap::DashMap;
use http::{Request, Response};
use hyper::body::Incoming;
use hyper::client::conn::http1;
use hyper_util::rt::TokioIo;
use std::sync::Arc;
use tokio::spawn;
use url::Url;

use hyper::client::conn::http2;

/// Type alias for H2 sender
type H2Sender = http2::SendRequest<http_body_util::Empty<bytes::Bytes>>;

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
                eprintln!("H1 Req error: {:?}", e);
                NetError::ConnectionClosed
            }),
            HttpStreamInner::H2(sender) => sender.send_request(req).await.map_err(|e| {
                eprintln!("H2 Req error: {:?}", e);
                NetError::ConnectionClosed
            }),
        }
    }
}

/// HTTP/2 session cache for multiplexing.
/// Stores active H2 senders by host:port key for reuse.
struct H2SessionCache {
    sessions: DashMap<String, H2Sender>,
}

impl H2SessionCache {
    fn new() -> Self {
        Self {
            sessions: DashMap::new(),
        }
    }

    /// Get session key from URL
    fn key(url: &Url) -> Option<String> {
        Some(format!(
            "{}:{}",
            url.host_str()?,
            url.port_or_known_default()?
        ))
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

    pub async fn create_stream(
        &self,
        url: &Url,
        proxy: Option<&crate::socket::proxy::ProxySettings>,
        h2_settings: Option<&crate::http::H2Settings>,
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
            // H2 Handshake with optional fingerprinting settings
            let settings = h2_settings.copied().unwrap_or_default();

            let (sender, conn) = http2::Builder::new(io::TokioExecutor::new())
                .initial_stream_window_size(settings.initial_window_size)
                .initial_connection_window_size(settings.initial_window_size)
                .max_frame_size(settings.max_frame_size)
                .max_concurrent_streams(settings.max_concurrent_streams)
                .max_header_list_size(settings.max_header_list_size)
                .handshake::<_, http_body_util::Empty<bytes::Bytes>>(io)
                .await
                .map_err(|e| {
                    eprintln!("H2 Handshake failed: {:?}", e);
                    NetError::ConnectionFailed
                })?;

            // Store sender in cache for multiplexing
            self.h2_cache.store(url, sender.clone());

            spawn(async move {
                if let Err(e) = conn.await {
                    eprintln!("H2 Connection failed: {:?}", e);
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
                    eprintln!("H1 Connection failed: {:?}", e);
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
    use hyper::rt::Executor;
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
