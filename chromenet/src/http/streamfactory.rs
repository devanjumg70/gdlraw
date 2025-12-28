//! HTTP Stream Factory
//!
//! Creates HTTP/1.1 and HTTP/2 streams for network transactions.
//! Supports H2 multiplexing and browser fingerprint emulation.

use crate::base::neterror::NetError;
use crate::http::h2fingerprint::H2Fingerprint;
use crate::socket::pool::{ClientSocketPool, PoolResult};
use bytes::Bytes;
use dashmap::DashMap;
use http::{Request, Response};
use http2::client;
use http2::RecvStream;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::client::conn::http1;
use hyper_util::rt::TokioIo;
use std::sync::Arc;
use tokio::spawn;
use url::Url;

/// Type alias for H2 sender (using http2 crate's forked h2)
/// Uses bytes::Bytes as the body type which implements Buf
type H2Sender = client::SendRequest<Bytes>;

/// HTTP response body enum that abstracts over H1 and H2 body types
pub enum StreamBody {
    H1(Incoming),
    H2(RecvStream),
}

/// Wraps the underlying protocol stream (H1/H2).
pub struct HttpStream {
    inner: HttpStreamInner,
    is_reused: bool,
}

enum HttpStreamInner {
    // H1 sender now uses Full<Bytes> for body support
    H1(http1::SendRequest<Full<Bytes>>),
    H2(H2Sender),
}

impl HttpStream {
    pub fn is_h2(&self) -> bool {
        matches!(self.inner, HttpStreamInner::H2(_))
    }

    pub fn is_reused(&self) -> bool {
        self.is_reused
    }

    /// Send an HTTP request with a body and get the response.
    ///
    /// For H1, uses hyper's body types with Full<Bytes>.
    /// For H2, uses http2 crate's API, sending body via SendStream if non-empty.
    pub async fn send_request(
        &mut self,
        req: Request<Full<Bytes>>,
    ) -> Result<Response<StreamBody>, NetError> {
        match &mut self.inner {
            HttpStreamInner::H1(sender) => {
                let resp = sender.send_request(req).await.map_err(|e| {
                    tracing::debug!("H1 request error: {:?}", e);
                    NetError::ConnectionClosed
                })?;
                Ok(resp.map(StreamBody::H1))
            }
            HttpStreamInner::H2(sender) => {
                // Clone sender because ready() consumes it
                let sender = sender.clone();

                // Wait for the connection to be ready
                let mut ready_sender = sender.ready().await.map_err(|e| {
                    tracing::debug!("H2 ready error: {:?}", e);
                    NetError::ConnectionFailed
                })?;

                // Extract body using BodyExt
                let (parts, body) = req.into_parts();
                let body_bytes = body.collect().await.map_err(|_| NetError::ConnectionClosed)?.to_bytes();
                let has_body = !body_bytes.is_empty();

                // Create H2 request
                let req_h2 = Request::from_parts(parts, ());

                // Send request - end_of_stream = true only if no body
                let (response_fut, mut send_stream) =
                    ready_sender.send_request(req_h2, !has_body).map_err(|e| {
                        tracing::debug!("H2 send_request error: {:?}", e);
                        NetError::ConnectionFailed
                    })?;

                // Send body data if present
                if has_body {
                    send_stream.send_data(body_bytes, true).map_err(|e| {
                        tracing::debug!("H2 send_data error: {:?}", e);
                        NetError::ConnectionFailed
                    })?;
                }

                // Await the response
                let resp = response_fut.await.map_err(|e| {
                    tracing::debug!("H2 response error: {:?}", e);
                    NetError::ConnectionClosed
                })?;

                // Convert to our response type
                let (parts, recv_stream) = resp.into_parts();
                Ok(Response::from_parts(parts, StreamBody::H2(recv_stream)))
            }
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
        Some(entry.value().clone())
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
            let mut builder = client::Builder::new();

            // Apply window sizes
            builder.initial_window_size(fp.initial_window_size);
            builder.initial_connection_window_size(fp.initial_conn_window_size);

            // Apply frame limits
            if let Some(max_frame) = fp.max_frame_size {
                builder.max_frame_size(max_frame);
            }
            if let Some(max_streams) = fp.max_concurrent_streams {
                builder.max_concurrent_streams(max_streams);
            }
            if let Some(max_header_list) = fp.max_header_list_size {
                builder.max_header_list_size(max_header_list);
            }
            if let Some(header_table_size) = fp.header_table_size {
                builder.header_table_size(header_table_size);
            }

            // Apply pseudo-header order (critical for fingerprinting)
            // Note: pseudo_order is set per-request, not on the connection builder
            // if let Some(order) = &fp.pseudo_order { ... }

            // Apply settings order (critical for fingerprinting)
            if let Some(order) = &fp.settings_order {
                builder.settings_order(order.clone());
            }

            // Apply priority frames (priorities sent after connection)
            // Note: priorities are sent asynchronously after handshake
            // if let Some(ref priorities) = fp.priorities { ... }

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

            // Perform handshake with Bytes body type
            let (sender, conn) = builder.handshake::<_, Bytes>(io).await.map_err(|e| {
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
