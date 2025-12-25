//! Response body streaming.
//! Mirrors Chromium's HttpStream::ReadResponseBody.

use crate::base::neterror::NetError;
use bytes::Bytes;
use hyper::body::Incoming;

/// Response body wrapper for streaming.
pub struct ResponseBody {
    inner: Incoming,
}

impl ResponseBody {
    /// Create a new response body wrapper.
    pub fn new(inner: Incoming) -> Self {
        Self { inner }
    }

    /// Read entire body as bytes.
    pub async fn bytes(self) -> Result<Bytes, NetError> {
        use http_body_util::BodyExt;
        let collected = self
            .inner
            .collect()
            .await
            .map_err(|_| NetError::HttpBodyError)?;
        Ok(collected.to_bytes())
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

    /// Get the inner Incoming body for low-level access.
    pub fn into_inner(self) -> Incoming {
        self.inner
    }
}
