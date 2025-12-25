use crate::base::neterror::NetError;
use crate::socket::pool::{ClientSocketPool, PoolResult};
use http::{Request, Response};
use hyper::body::Incoming;
use hyper::client::conn::http1;
use hyper_util::rt::TokioIo;
use std::sync::Arc;
use tokio::spawn;
use url::Url;

use hyper::client::conn::http2;

/// Wraps the underlying protocol stream (H1/H2).
pub struct HttpStream {
    inner: HttpStreamInner,
    is_reused: bool,
}

enum HttpStreamInner {
    H1(http1::SendRequest<http_body_util::Empty<bytes::Bytes>>),
    H2(http2::SendRequest<http_body_util::Empty<bytes::Bytes>>),
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

pub struct HttpStreamFactory {
    pool: Arc<ClientSocketPool>,
}

impl HttpStreamFactory {
    pub fn new(pool: Arc<ClientSocketPool>) -> Self {
        Self { pool }
    }

    pub async fn create_stream(
        &self,
        url: &Url,
        proxy: Option<&crate::socket::proxy::ProxySettings>,
        h2_settings: Option<&crate::http::H2Settings>,
    ) -> Result<HttpStream, NetError> {
        // 1. Get socket from pool
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

            spawn(async move {
                if let Err(e) = conn.await {
                    eprintln!("H2 Connection failed: {:?}", e);
                }
            });

            Ok(HttpStream { inner: HttpStreamInner::H2(sender), is_reused: pool_result.is_reused })
        } else {
            // H1 Handshake (Default)
            let (sender, conn) =
                http1::handshake(io).await.map_err(|_| NetError::ConnectionFailed)?;

            spawn(async move {
                if let Err(e) = conn.await {
                    eprintln!("H1 Connection failed: {:?}", e);
                }
            });

            Ok(HttpStream { inner: HttpStreamInner::H1(sender), is_reused: pool_result.is_reused })
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
