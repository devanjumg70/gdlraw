# WebSocket Module

Full WebSocket client with tokio-tungstenite integration.

## Quick Start

```rust
use chromenet::ws::{WebSocket, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect
    let ws = WebSocket::connect("wss://echo.websocket.org").await?;
    
    // Send text
    ws.send_text("Hello!").await?;
    
    // Send binary
    ws.send_binary(vec![1, 2, 3]).await?;
    
    // Receive
    while let Some(msg) = ws.recv().await? {
        match msg {
            Message::Text(s) => println!("Text: {}", s),
            Message::Binary(b) => println!("Binary: {} bytes", b.len()),
            Message::Close(_) => break,
            _ => {}
        }
    }
    
    ws.close(None).await?;
    Ok(())
}
```

## Types

### WebSocket
Main connection type with send/receive methods.

| Method | Description |
|--------|-------------|
| `connect(url)` | Connect to server |
| `send(msg)` | Send any Message type |
| `send_text(s)` | Send text message |
| `send_binary(b)` | Send binary data |
| `recv()` | Receive next message |
| `ping(data)` | Send ping frame |
| `close(frame)` | Close connection |

### Message
WebSocket message types.

```rust
pub enum Message {
    Text(String),
    Binary(Bytes),
    Ping(Vec<u8>),
    Pong(Vec<u8>),
    Close(Option<CloseFrame>),
}
```

### WebSocketBuilder
Builder pattern for connection configuration.

```rust
let ws = WebSocketBuilder::new()
    .url("wss://example.com/ws")?
    .header("Authorization", "Bearer token")
    .subprotocol("graphql-ws")
    .connect()
    .await?;
```

## Close Codes

```rust
CloseCode::NORMAL         // 1000
CloseCode::GOING_AWAY     // 1001
CloseCode::PROTOCOL_ERROR // 1002
CloseCode::ABNORMAL       // 1006
```
