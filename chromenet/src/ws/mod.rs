//! WebSocket client support.
//!
//! Provides WebSocket connections using tokio-tungstenite with boring TLS.
//! Mirrors Chromium's net/websockets/ implementation pattern.
//!
//! # Example
//! ```ignore
//! use chromenet::ws::{WebSocket, Message};
//!
//! let ws = WebSocket::connect("wss://echo.websocket.org").await?;
//! ws.send(Message::Text("Hello".into())).await?;
//! let msg = ws.recv().await?;
//! ```

mod connection;
mod message;

pub use connection::WebSocket;
pub use message::Message;
