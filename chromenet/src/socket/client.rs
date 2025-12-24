use std::fmt;
use std::io::ErrorKind;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite};

/// Represents a connected socket (TCP or SSL).
/// Mimics net::StreamSocket.
pub trait StreamSocket: AsyncRead + AsyncWrite + Unpin + Send + Sync + fmt::Debug {
    /// Returns true if the socket is still connected.
    /// Note: This does a non-blocking check, not a full liveness probe.
    fn is_connected(&self) -> bool;

    /// Returns true if the socket is connected and has no pending data.
    /// Matches Chromium's IsConnectedAndIdle().
    fn is_connected_and_idle(&self) -> bool;
}

#[derive(Debug)]
pub enum SocketType {
    Tcp(tokio::net::TcpStream),
    Ssl(tokio_boring::SslStream<tokio::net::TcpStream>),
}

impl SocketType {
    /// Check if the underlying TCP socket is still connected.
    /// Uses peer_addr() check as a lightweight liveness test.
    fn check_tcp_connected(stream: &tokio::net::TcpStream) -> bool {
        // peer_addr() returns Err if socket is disconnected
        if stream.peer_addr().is_err() {
            return false;
        }

        // Try a non-blocking peek to detect closed connections
        // This catches RST and FIN conditions
        let mut buf = [0u8; 1];
        match stream.try_read(&mut buf) {
            Ok(0) => false,                                          // EOF - connection closed
            Ok(_) => true, // Data available, still connected
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => true, // No data, but connected
            Err(_) => false, // Error - assume disconnected
        }
    }

    /// Check if the SSL stream is still connected.
    fn check_ssl_connected(stream: &tokio_boring::SslStream<tokio::net::TcpStream>) -> bool {
        // Check underlying TCP stream
        Self::check_tcp_connected(stream.get_ref())
    }
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
        match self {
            SocketType::Tcp(s) => Self::check_tcp_connected(s),
            SocketType::Ssl(s) => Self::check_ssl_connected(s),
        }
    }

    fn is_connected_and_idle(&self) -> bool {
        // For now, same as is_connected - we don't track pending data
        // In a full impl, we'd check if there's unread data in buffers
        self.is_connected()
    }
}
