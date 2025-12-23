use crate::base::neterror::NetError;
use crate::socket::pool::ClientSocketPool;
use http::{Request, Response};
use hyper::body::Incoming;
use hyper::client::conn::http1;
use hyper_util::rt::TokioIo;
use std::sync::Arc;
use tokio::spawn;
use url::Url;

/// Wraps the underlying protocol stream (H1/H2).
/// Equivalent to net::HttpStream.
pub struct HttpStream {
    sender: http1::SendRequest<http_body_util::Empty<bytes::Bytes>>,
    // We only support Sending Empty bodies for GET in this MVP for simplicity
    // To support bodies, we need a generic body type
}

impl HttpStream {
    pub async fn send_request(
        &mut self,
        req: Request<http_body_util::Empty<bytes::Bytes>>,
    ) -> Result<Response<Incoming>, NetError> {
        self.sender.send_request(req).await.map_err(|e| {
            eprintln!("Req error: {:?}", e);
            NetError::ConnectionClosed
        })
    }
}

pub struct HttpStreamFactory {
    pool: Arc<ClientSocketPool>,
}

impl HttpStreamFactory {
    pub fn new(pool: Arc<ClientSocketPool>) -> Self {
        Self { pool }
    }

    pub async fn request_stream(&self, url: &Url) -> Result<HttpStream, NetError> {
        // 1. Get raw socket
        let socket = self.pool.request_socket(url).await?;

        // 2. Handshake (Only HTTP/1.1 for this MVP "Raw" step)
        let io = TokioIo::new(socket);

        let (sender, conn) = http1::handshake(io).await.map_err(|_| NetError::ConnectionFailed)?;

        // 3. Spawn the connection driver
        spawn(async move {
            if let Err(e) = conn.await {
                eprintln!("Connection failed: {:?}", e);
            }
        });

        Ok(HttpStream { sender })
    }
}
