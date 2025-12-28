//! WebSocket connection with tokio-tungstenite.
//!
//! Provides full WebSocket client functionality.

use super::message::{CloseCode, CloseFrame, Message};
use crate::base::neterror::NetError;
use bytes::Bytes;
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite, MaybeTlsStream, WebSocketStream};
use url::Url;

/// Type alias for the WebSocket stream.
type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// WebSocket connection.
///
/// Thread-safe wrapper around a WebSocket stream with send/recv methods.
pub struct WebSocket {
    sink: Arc<Mutex<SplitSink<WsStream, tungstenite::Message>>>,
    stream: Arc<Mutex<SplitStream<WsStream>>>,
    url: Url,
}

impl WebSocket {
    /// Connect to a WebSocket server.
    ///
    /// # Example
    /// ```ignore
    /// let ws = WebSocket::connect("wss://echo.websocket.org").await?;
    /// ```
    pub async fn connect(url: &str) -> Result<Self, NetError> {
        let url = Url::parse(url).map_err(|_| NetError::InvalidUrl)?;

        // Validate scheme
        if url.scheme() != "ws" && url.scheme() != "wss" {
            return Err(NetError::InvalidUrl);
        }

        let (ws_stream, _response) = connect_async(url.as_str()).await.map_err(|e| {
            tracing::debug!("WebSocket connect error: {:?}", e);
            NetError::ConnectionFailed
        })?;

        let (sink, stream) = ws_stream.split();

        Ok(Self {
            sink: Arc::new(Mutex::new(sink)),
            stream: Arc::new(Mutex::new(stream)),
            url,
        })
    }

    /// Get the URL this WebSocket is connected to.
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Send a message.
    pub async fn send(&self, msg: Message) -> Result<(), NetError> {
        let tung_msg = message_to_tungstenite(msg);
        let mut sink = self.sink.lock().await;
        sink.send(tung_msg).await.map_err(|e| {
            tracing::debug!("WebSocket send error: {:?}", e);
            NetError::ConnectionClosed
        })
    }

    /// Send a text message.
    pub async fn send_text(&self, text: impl Into<String>) -> Result<(), NetError> {
        self.send(Message::Text(text.into())).await
    }

    /// Send binary data.
    pub async fn send_binary(&self, data: impl Into<Bytes>) -> Result<(), NetError> {
        self.send(Message::Binary(data.into())).await
    }

    /// Receive a message.
    ///
    /// Returns `None` if the connection is closed.
    pub async fn recv(&self) -> Result<Option<Message>, NetError> {
        let mut stream = self.stream.lock().await;
        match stream.next().await {
            Some(Ok(msg)) => Ok(Some(tungstenite_to_message(msg))),
            Some(Err(e)) => {
                tracing::debug!("WebSocket recv error: {:?}", e);
                Err(NetError::ConnectionClosed)
            }
            None => Ok(None),
        }
    }

    /// Close the connection with optional code and reason.
    pub async fn close(&self, frame: Option<CloseFrame>) -> Result<(), NetError> {
        let msg = Message::Close(frame);
        self.send(msg).await
    }

    /// Ping the server.
    pub async fn ping(&self, data: Vec<u8>) -> Result<(), NetError> {
        self.send(Message::Ping(data)).await
    }
}

/// WebSocket connection builder.
#[derive(Debug, Clone)]
pub struct WebSocketBuilder {
    url: Option<Url>,
    headers: http::HeaderMap,
    subprotocols: Vec<String>,
}

impl Default for WebSocketBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl WebSocketBuilder {
    /// Create a new WebSocket builder.
    pub fn new() -> Self {
        Self {
            url: None,
            headers: http::HeaderMap::new(),
            subprotocols: Vec::new(),
        }
    }

    /// Set the URL to connect to.
    pub fn url(mut self, url: &str) -> Result<Self, NetError> {
        let url = Url::parse(url).map_err(|_| NetError::InvalidUrl)?;

        // Validate scheme
        if url.scheme() != "ws" && url.scheme() != "wss" {
            return Err(NetError::InvalidUrl);
        }

        self.url = Some(url);
        Ok(self)
    }

    /// Add a header to the WebSocket handshake.
    pub fn header(mut self, name: &str, value: &str) -> Self {
        if let (Ok(name), Ok(value)) = (
            http::header::HeaderName::try_from(name),
            http::header::HeaderValue::try_from(value),
        ) {
            self.headers.insert(name, value);
        }
        self
    }

    /// Add a subprotocol.
    pub fn subprotocol(mut self, protocol: impl Into<String>) -> Self {
        self.subprotocols.push(protocol.into());
        self
    }

    /// Get the URL if set.
    pub fn get_url(&self) -> Option<&Url> {
        self.url.as_ref()
    }

    /// Get the headers.
    pub fn get_headers(&self) -> &http::HeaderMap {
        &self.headers
    }

    /// Check if secure (wss://).
    pub fn is_secure(&self) -> bool {
        self.url.as_ref().is_some_and(|u| u.scheme() == "wss")
    }

    /// Connect to the server.
    pub async fn connect(self) -> Result<WebSocket, NetError> {
        let url = self.url.ok_or(NetError::InvalidUrl)?;
        WebSocket::connect(url.as_str()).await
    }
}

/// Convert our Message to tungstenite Message.
fn message_to_tungstenite(msg: Message) -> tungstenite::Message {
    match msg {
        Message::Text(s) => tungstenite::Message::Text(s),
        Message::Binary(b) => tungstenite::Message::Binary(b.to_vec()),
        Message::Ping(d) => tungstenite::Message::Ping(d),
        Message::Pong(d) => tungstenite::Message::Pong(d),
        Message::Close(frame) => {
            let tung_frame = frame.map(|f| tungstenite::protocol::CloseFrame {
                code: tungstenite::protocol::frame::coding::CloseCode::from(f.code.0),
                reason: f.reason.into(),
            });
            tungstenite::Message::Close(tung_frame)
        }
    }
}

/// Convert tungstenite Message to our Message.
fn tungstenite_to_message(msg: tungstenite::Message) -> Message {
    match msg {
        tungstenite::Message::Text(s) => Message::Text(s.to_string()),
        tungstenite::Message::Binary(b) => Message::Binary(Bytes::from(b.to_vec())),
        tungstenite::Message::Ping(d) => Message::Ping(d.to_vec()),
        tungstenite::Message::Pong(d) => Message::Pong(d.to_vec()),
        tungstenite::Message::Close(frame) => {
            let our_frame = frame.map(|f| CloseFrame {
                code: CloseCode(f.code.into()),
                reason: f.reason.to_string(),
            });
            Message::Close(our_frame)
        }
        tungstenite::Message::Frame(_) => Message::Binary(Bytes::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_new() {
        let builder = WebSocketBuilder::new();
        assert!(builder.url.is_none());
    }

    #[test]
    fn test_builder_url() {
        let builder = WebSocketBuilder::new().url("ws://example.com/ws").unwrap();
        assert!(builder.url.is_some());
        assert!(!builder.is_secure());
    }

    #[test]
    fn test_builder_secure() {
        let builder = WebSocketBuilder::new().url("wss://example.com/ws").unwrap();
        assert!(builder.is_secure());
    }

    #[test]
    fn test_builder_invalid_scheme() {
        let result = WebSocketBuilder::new().url("http://example.com");
        assert!(result.is_err());
    }

    #[test]
    fn test_builder_headers() {
        let builder = WebSocketBuilder::new().header("Authorization", "Bearer token");
        assert!(builder.headers.contains_key("authorization"));
    }

    #[test]
    fn test_builder_subprotocol() {
        let builder = WebSocketBuilder::new()
            .subprotocol("graphql-ws")
            .subprotocol("protocol2");
        assert_eq!(builder.subprotocols.len(), 2);
    }

    #[test]
    fn test_message_conversion() {
        // Text
        let msg = Message::Text("hello".into());
        let tung = message_to_tungstenite(msg.clone());
        let back = tungstenite_to_message(tung);
        assert!(matches!(back, Message::Text(s) if s == "hello"));

        // Binary
        let msg = Message::Binary(Bytes::from_static(b"data"));
        let tung = message_to_tungstenite(msg);
        let back = tungstenite_to_message(tung);
        assert!(matches!(back, Message::Binary(b) if b == Bytes::from_static(b"data")));
    }
}
