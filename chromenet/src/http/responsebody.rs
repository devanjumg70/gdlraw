//! Response body streaming.
//! Mirrors Chromium's HttpStream::ReadResponseBody.

use crate::base::neterror::NetError;
use crate::http::streamfactory::StreamBody;
use bytes::Bytes;
use http2::RecvStream;
use hyper::body::Incoming;
use std::pin::Pin;
use std::task::{Context, Poll};

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
    ///
    /// Note: This collects the entire body into memory.
    /// For large responses, use `stream()` instead.
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

    /// Convert into a stream of byte chunks.
    ///
    /// Use this for memory-efficient streaming of large responses.
    ///
    /// # Example
    /// ```ignore
    /// use futures::StreamExt;
    /// let mut stream = body.into_stream();
    /// while let Some(chunk) = stream.next().await {
    ///     let bytes = chunk?;
    ///     // Process chunk
    /// }
    /// ```
    pub fn into_stream(self) -> BodyStream {
        BodyStream { inner: self }
    }
}

/// Async stream wrapper for ResponseBody.
///
/// Implements `futures::Stream` for chunk-by-chunk reading.
pub struct BodyStream {
    inner: ResponseBody,
}

impl futures::Stream for BodyStream {
    type Item = Result<Bytes, NetError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match &mut self.inner {
            ResponseBody::H1(incoming) => {
                use http_body::Body;
                match Pin::new(incoming).poll_frame(cx) {
                    Poll::Ready(Some(Ok(frame))) => {
                        if let Some(data) = frame.data_ref() {
                            Poll::Ready(Some(Ok(data.clone())))
                        } else {
                            // Trailers frame, continue polling
                            cx.waker().wake_by_ref();
                            Poll::Pending
                        }
                    }
                    Poll::Ready(Some(Err(_))) => Poll::Ready(Some(Err(NetError::HttpBodyError))),
                    Poll::Ready(None) => Poll::Ready(None),
                    Poll::Pending => Poll::Pending,
                }
            }
            ResponseBody::H2(recv_stream) => {
                // For H2, we need to poll the recv_stream
                // The http2 crate's RecvStream requires different handling
                match Pin::new(recv_stream).poll_data(cx) {
                    Poll::Ready(Some(Ok(data))) => Poll::Ready(Some(Ok(data))),
                    Poll::Ready(Some(Err(_))) => Poll::Ready(Some(Err(NetError::HttpBodyError))),
                    Poll::Ready(None) => Poll::Ready(None),
                    Poll::Pending => Poll::Pending,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests would require mocking HTTP responses
    // which is complex, so we test the type structure
    #[test]
    fn test_body_stream_type() {
        // Verify BodyStream implements Stream
        fn assert_stream<S: futures::Stream>() {}
        assert_stream::<BodyStream>();
    }
}
