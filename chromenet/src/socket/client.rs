use std::fmt;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite};

/// Represents a connected socket (TCP or SSL).
/// Mimics net::StreamSocket.
pub trait StreamSocket: AsyncRead + AsyncWrite + Unpin + Send + Sync + fmt::Debug {
    fn is_connected(&self) -> bool;
}

// Wrapper for TcpStream to implement StreamSocket?
// Or we just use `Box<dyn StreamSocket>`?

#[derive(Debug)]
pub enum SocketType {
    Tcp(tokio::net::TcpStream),
    Ssl(tokio_boring::SslStream<tokio::net::TcpStream>),
}

impl AsyncRead for SocketType {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            SocketType::Tcp(s) => Pin::new(s).poll_read(cx, buf),
            SocketType::Ssl(s) => Pin::new(s).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for SocketType {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        match self.get_mut() {
            SocketType::Tcp(s) => Pin::new(s).poll_write(cx, buf),
            SocketType::Ssl(s) => Pin::new(s).poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            SocketType::Tcp(s) => Pin::new(s).poll_flush(cx),
            SocketType::Ssl(s) => Pin::new(s).poll_flush(cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.get_mut() {
            SocketType::Tcp(s) => Pin::new(s).poll_shutdown(cx),
            SocketType::Ssl(s) => Pin::new(s).poll_shutdown(cx),
        }
    }
}

impl StreamSocket for SocketType {
    fn is_connected(&self) -> bool {
        // Simple check, real impl might read 1 byte with peek
        true
    }
}
