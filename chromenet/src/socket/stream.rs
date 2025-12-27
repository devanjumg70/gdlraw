//! Socket abstraction for TLS-in-TLS and polymorphic socket handling.
//!
//! This module provides a `StreamSocket` trait that allows uniform handling of
//! different socket types: plain TCP, TLS over TCP, and nested TLS (TLS-in-TLS).
//!
//! Based on Chromium's `StreamSocket` interface which provides polymorphism
//! for `TcpClientSocket`, `SSLClientSocket`, and nested tunnel sockets.

use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;
use tokio_boring::SslStream;

/// A trait for any socket that supports async read/write operations.
/// Enables TLS wrapping of any socket type (TCP, TLS, or nested TLS).
///
/// Chromium equivalent: `net::StreamSocket`
pub trait StreamSocket: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static {
    /// Check if the socket is connected.
    fn is_connected(&self) -> bool {
        true
    }
}

// Implement StreamSocket for TcpStream
impl StreamSocket for TcpStream {}

// Implement StreamSocket for SslStream<T> where T is any StreamSocket
impl<S: StreamSocket> StreamSocket for SslStream<S> {}

/// A wrapper type for boxed dynamic StreamSocket that is object-safe.
/// This avoids conflicting trait implementations with tokio's blanket impls.
pub struct BoxedSocket {
    inner: Pin<Box<dyn StreamSocket>>,
}

impl BoxedSocket {
    /// Create a new BoxedSocket from any StreamSocket.
    pub fn new<S: StreamSocket>(socket: S) -> Self {
        Self {
            inner: Box::pin(socket),
        }
    }

    /// Get a pinned mutable reference to the inner socket.
    pub fn as_mut(&mut self) -> Pin<&mut dyn StreamSocket> {
        self.inner.as_mut()
    }

    /// Check if the socket is connected.
    pub fn is_connected(&self) -> bool {
        // Delegate to the inner StreamSocket trait method
        // Note: For dyn trait objects, we just assume connected
        // Full implementation would require downcast or non-object-safe trait
        true
    }
}

impl AsyncRead for BoxedSocket {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        self.inner.as_mut().poll_read(cx, buf)
    }
}

impl AsyncWrite for BoxedSocket {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        self.inner.as_mut().poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        self.inner.as_mut().poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        self.inner.as_mut().poll_shutdown(cx)
    }
}

// BoxedSocket is Unpin because it's a wrapper that handles pinning internally
impl Unpin for BoxedSocket {}

// Implement hyper::rt::Read and hyper::rt::Write for BoxedSocket
// This allows http2 crate to use BoxedSocket directly
impl hyper::rt::Read for BoxedSocket {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut buf: hyper::rt::ReadBufCursor<'_>,
    ) -> Poll<std::io::Result<()>> {
        // SAFETY: We're using the hyper ReadBufCursor correctly
        let unfilled = unsafe { buf.as_mut() };
        let mut read_buf = ReadBuf::uninit(unfilled);
        // Disambiguate by calling tokio AsyncRead directly
        match AsyncRead::poll_read(self.as_mut(), cx, &mut read_buf) {
            Poll::Ready(Ok(())) => {
                let n = read_buf.filled().len();
                unsafe { buf.advance(n) };
                Poll::Ready(Ok(()))
            }
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl hyper::rt::Write for BoxedSocket {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        <Self as AsyncWrite>::poll_write(self, cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        <Self as AsyncWrite>::poll_flush(self, cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        <Self as AsyncWrite>::poll_shutdown(self, cx)
    }
}
