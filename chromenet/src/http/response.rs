//! HTTP Response with body access.

use crate::http::ResponseBody;
use http::{HeaderMap, StatusCode, Version};
use hyper::body::Incoming;

/// HTTP Response with accessible body.
/// This is the user-facing response type that owns the body.
pub struct HttpResponse {
    status: StatusCode,
    version: Version,
    headers: HeaderMap,
    body: Option<ResponseBody>,
}

impl HttpResponse {
    /// Create from hyper Response<Incoming>.
    pub fn from_hyper(resp: http::Response<Incoming>) -> Self {
        let (parts, body) = resp.into_parts();
        Self {
            status: parts.status,
            version: parts.version,
            headers: parts.headers,
            body: Some(ResponseBody::new(body)),
        }
    }

    /// Get the status code.
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// Get the HTTP version.
    pub fn version(&self) -> Version {
        self.version
    }

    /// Get a reference to the headers.
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Take the response body for consumption.
    /// Can only be called once - subsequent calls return None.
    pub fn take_body(&mut self) -> Option<ResponseBody> {
        self.body.take()
    }

    /// Convenience method to consume body as bytes.
    pub async fn bytes(mut self) -> Result<bytes::Bytes, crate::base::neterror::NetError> {
        self.body
            .take()
            .ok_or(crate::base::neterror::NetError::HttpBodyError)?
            .bytes()
            .await
    }

    /// Convenience method to consume body as text.
    pub async fn text(mut self) -> Result<String, crate::base::neterror::NetError> {
        self.body
            .take()
            .ok_or(crate::base::neterror::NetError::HttpBodyError)?
            .text()
            .await
    }

    /// Convenience method to consume body as JSON.
    pub async fn json<T: serde::de::DeserializeOwned>(
        mut self,
    ) -> Result<T, crate::base::neterror::NetError> {
        self.body
            .take()
            .ok_or(crate::base::neterror::NetError::HttpBodyError)?
            .json()
            .await
    }
}
