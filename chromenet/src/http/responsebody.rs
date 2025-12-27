//! Response body streaming.
//! Mirrors Chromium's HttpStream::ReadResponseBody.

use crate::base::neterror::NetError;
use crate::http::streamfactory::StreamBody;
use bytes::Bytes;
use http2::RecvStream;
use hyper::body::Incoming;

/// Response body wrapper for streaming.
/// Supports both HTTP/1.1 (hyper Incoming) and HTTP/2 (http2 RecvStream).
pub enum ResponseBody {
    H1(Incoming),
    H2(RecvStream),
}

impl ResponseBody {
    /// Create a new response body wrapper from hyper Incoming.
    pub fn new(inner: Incoming) -> Self {
        ResponseBody::H1(inner)
    }

    /// Create from StreamBody enum.
    pub fn from_stream(stream: StreamBody) -> Self {
        match stream {
            StreamBody::H1(incoming) => ResponseBody::H1(incoming),
            StreamBody::H2(recv) => ResponseBody::H2(recv),
        }
    }

    /// Read entire body as bytes.
    pub async fn bytes(self) -> Result<Bytes, NetError> {
        match self {
            ResponseBody::H1(incoming) => {
                use http_body_util::BodyExt;
                let collected = incoming
                    .collect()
                    .await
                    .map_err(|_| NetError::HttpBodyError)?;
                Ok(collected.to_bytes())
            }
            ResponseBody::H2(mut recv_stream) => {
                use bytes::BufMut;
                let mut data = bytes::BytesMut::new();
                while let Some(chunk) = recv_stream.data().await {
                    let chunk = chunk.map_err(|_| NetError::HttpBodyError)?;
                    data.put(chunk);
                }
                Ok(data.freeze())
            }
        }
    }

    /// Read body as UTF-8 string.
    pub async fn text(self) -> Result<String, NetError> {
        let bytes = self.bytes().await?;
        String::from_utf8(bytes.to_vec()).map_err(|_| NetError::InvalidUtf8)
    }

    /// Read body as JSON, deserializing to type T.
    pub async fn json<T: serde::de::DeserializeOwned>(self) -> Result<T, NetError> {
        let bytes = self.bytes().await?;
        serde_json::from_slice(&bytes).map_err(|_| NetError::JsonParseError)
    }
}
