# chromenet

A production-ready Rust HTTP client library with Chromium-inspired architecture for browser fingerprinting, TLS emulation, and advanced networking features.

## Features

### Core Networking
- **Connection Pooling**: Chromium-style 6/host, 256 total socket limits
- **HTTP/1.1 & HTTP/2**: Full protocol support with multiplexing
- **HTTP Cache**: In-memory with Cache-Control, ETags, LRU eviction
- **Streaming**: Memory-efficient large body streaming via `futures::Stream`

### Browser Emulation
- **TLS Fingerprinting**: 30+ BoringSSL options for precise fingerprint control
- **HTTP/2 Fingerprinting**: Settings order, pseudo-header order, priority frames
- **67 Browser Profiles**: Chrome, Firefox, Safari, Edge, OkHttp, Opera
- **Device Emulation**: User-Agent, Client Hints, screen dimensions

### Cookies & Security
- **Cookie Management**: RFC 6265 compliant CookieMonster with PSL validation
- **HSTS**: Preload list + dynamic headers with JSON persistence
- **Certificate Pinning**: SPKI hash verification

### Advanced Features
- **WebSocket**: Full client with tokio-tungstenite (`send`, `recv`, `ping`, `close`)
- **Multipart**: Form uploads with automatic boundary generation
- **Proxy Support**: HTTP, HTTPS, SOCKS5 with rotation and NO_PROXY rules
- **QUIC/HTTP3**: Type structure ready for quinn integration

## Quick Start

```rust
use chromenet::client::Client;
use chromenet::emulation::profiles::chrome::Chrome;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .emulation(Chrome::V143)
        .build();

    let response = client
        .get("https://httpbin.org/get")
        .send()
        .await?;

    println!("Status: {}", response.status());
    let text = response.text().await?;
    println!("Body: {}", text);
    Ok(())
}
```

## WebSocket Example

```rust
use chromenet::ws::{WebSocket, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ws = WebSocket::connect("wss://echo.websocket.org").await?;
    
    ws.send_text("Hello WebSocket!").await?;
    
    if let Some(msg) = ws.recv().await? {
        println!("Received: {:?}", msg);
    }
    
    ws.close(None).await?;
    Ok(())
}
```

## Multipart Upload

```rust
use chromenet::http::multipart::{Form, Part};

let form = Form::new()
    .text("username", "user123")
    .part("avatar", Part::bytes(image_data)
        .file_name("avatar.png")
        .content_type("image/png"));

let body = form.into_body();
let content_type = form.content_type();
```

## Installation

```toml
[dependencies]
chromenet = { path = "." }
tokio = { version = "1", features = ["full"] }
```

## Architecture

```
Client → RequestBuilder → URLRequestHttpJob → HttpNetworkTransaction
   → HttpStreamFactory → ClientSocketPool → ConnectJob → TLS/H2

Modules:
├── base/       # NetError, LoadState
├── cookies/    # CookieMonster, PSL, browser import
├── http/       # Transaction, Cache, Multipart, Streaming
├── socket/     # Pool, Proxy, ConnectJob
├── tls/        # HSTS, Pinning, CT
├── ws/         # WebSocket client
├── quic/       # HTTP/3 types
└── emulation/  # Browser profiles, device registry
```

## Browser Profiles

| Browser | Versions | Profiles |
|---------|----------|----------|
| Chrome  | V100-V143 | 21 |
| Firefox | V109-V145 | 13 |
| Safari  | V15-V18   | 15 |
| Edge    | V101-V142 | 10 |
| OkHttp  | 3.x-5.x   | 8 |

## Testing

```bash
cargo test                    # Run 210+ unit tests
cargo test --ignored          # Include network tests
cargo bench                   # Run benchmarks
```

## Documentation

See the `docs/` directory for detailed module documentation:
- `architecture.md` - System design
- `cookies.md` - Cookie management
- `http.md` - HTTP transactions
- `socket.md` - Connection pooling
- `tls.md` - TLS security

## License

MIT License
