//! Request body for POST/PUT operations.
//!
//! Chromium mapping: net/base/upload_data_stream.h

use bytes::Bytes;
use http_body_util::Full;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Request body for HTTP methods that send data.
///
/// Supports in-memory bytes. Streaming body support can be added later.
#[derive(Debug, Clone, Default)]
pub enum RequestBody {
    /// No body (GET, HEAD, DELETE).
    #[default]
    Empty,
    /// Body with raw bytes.
    Bytes(Bytes),
}

impl From<String> for RequestBody {
    fn from(s: String) -> Self {
        RequestBody::Bytes(Bytes::from(s))
    }
}

impl From<Vec<u8>> for RequestBody {
    fn from(v: Vec<u8>) -> Self {
        RequestBody::Bytes(Bytes::from(v))
    }
}

impl From<&str> for RequestBody {
    fn from(s: &str) -> Self {
        RequestBody::Bytes(Bytes::from(s.to_owned()))
    }
}

impl From<Bytes> for RequestBody {
    fn from(b: Bytes) -> Self {
        RequestBody::Bytes(b)
    }
}

impl From<&[u8]> for RequestBody {
    fn from(b: &[u8]) -> Self {
        RequestBody::Bytes(Bytes::copy_from_slice(b))
    }
}

impl RequestBody {
    /// Check if the body is empty.
    pub fn is_empty(&self) -> bool {
        matches!(self, RequestBody::Empty)
    }

    /// Get the length of the body in bytes.
    pub fn len(&self) -> usize {
        match self {
            RequestBody::Empty => 0,
            RequestBody::Bytes(b) => b.len(),
        }
    }

    /// Take the inner bytes, consuming the body.
    pub fn take_bytes(&mut self) -> Bytes {
        match std::mem::take(self) {
            RequestBody::Empty => Bytes::new(),
            RequestBody::Bytes(b) => b,
        }
    }

    /// Convert to a Full<Bytes> for hyper compatibility.
    pub fn into_full(self) -> Full<Bytes> {
        match self {
            RequestBody::Empty => Full::new(Bytes::new()),
            RequestBody::Bytes(b) => Full::new(b),
        }
    }
}

/// Wrapper for RequestBody that implements http_body::Body trait.
pub struct BodyWrapper {
    inner: Option<Bytes>,
}

impl From<RequestBody> for BodyWrapper {
    fn from(body: RequestBody) -> Self {
        match body {
            RequestBody::Empty => BodyWrapper { inner: None },
            RequestBody::Bytes(b) => BodyWrapper { inner: Some(b) },
        }
    }
}

impl http_body::Body for BodyWrapper {
    type Data = Bytes;
    type Error = std::convert::Infallible;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        match self.inner.take() {
            Some(data) if !data.is_empty() => Poll::Ready(Some(Ok(http_body::Frame::data(data)))),
            _ => Poll::Ready(None),
        }
    }

    fn is_end_stream(&self) -> bool {
        self.inner.as_ref().is_none_or(|b| b.is_empty())
    }

    fn size_hint(&self) -> http_body::SizeHint {
        let size = self.inner.as_ref().map_or(0, |b| b.len() as u64);
        http_body::SizeHint::with_exact(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_body() {
        let body = RequestBody::Empty;
        assert!(body.is_empty());
        assert_eq!(body.len(), 0);
    }

    #[test]
    fn test_bytes_body() {
        let body = RequestBody::Bytes(Bytes::from("hello"));
        assert!(!body.is_empty());
        assert_eq!(body.len(), 5);
    }

    #[test]
    fn test_from_string() {
        let body: RequestBody = "hello world".to_string().into();
        assert_eq!(body.len(), 11);
    }

    #[test]
    fn test_from_str() {
        let body: RequestBody = "test".into();
        assert_eq!(body.len(), 4);
    }

    #[test]
    fn test_from_vec() {
        let body: RequestBody = vec![1u8, 2, 3, 4].into();
        assert_eq!(body.len(), 4);
    }

    #[test]
    fn test_from_bytes() {
        let body: RequestBody = Bytes::from_static(b"raw").into();
        assert_eq!(body.len(), 3);
    }

    #[test]
    fn test_default_is_empty() {
        let body = RequestBody::default();
        assert!(body.is_empty());
    }

    #[test]
    fn test_clone() {
        let body1: RequestBody = "data".into();
        let body2 = body1.clone();
        assert_eq!(body1.len(), body2.len());
    }

    #[test]
    fn test_into_full() {
        let body: RequestBody = "hello".into();
        let full = body.into_full();
        use http_body::Body;
        assert_eq!(full.size_hint().exact(), Some(5));
    }

    #[test]
    fn test_body_wrapper_size_hint() {
        use http_body::Body;

        let wrapper: BodyWrapper = RequestBody::Bytes(Bytes::from("test")).into();
        assert_eq!(wrapper.size_hint().exact(), Some(4));

        let empty_wrapper: BodyWrapper = RequestBody::Empty.into();
        assert_eq!(empty_wrapper.size_hint().exact(), Some(0));
    }
}
