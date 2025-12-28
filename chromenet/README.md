# chromenet

A Rust HTTP client library with Chromium-inspired architecture for browser fingerprinting and TLS emulation.

## Features

- **TLS Fingerprinting**: 30+ BoringSSL options for precise TLS fingerprint control
- **HTTP/2 Fingerprinting**: Settings order, pseudo-header order, priority frames
- **67 Browser Profiles**: Chrome, Firefox, Safari, Edge, OkHttp, Opera
- **Connection Pooling**: Chromium-style 6/host, 256 total socket limits
- **Cookie Management**: RFC 6265 compliant CookieMonster
- **Proxy Support**: HTTP, HTTPS, SOCKS5 with rotation and NO_PROXY rules

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

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
chromenet = { path = "." }
tokio = { version = "1", features = ["full"] }
```

## Architecture

```
Client → RequestBuilder → URLRequestHttpJob → HttpNetworkTransaction
   → HttpStreamFactory → ClientSocketPool → ConnectJob → TLS/H2
```

## Browser Profiles

| Browser | Versions | Total |
|---------|----------|-------|
| Chrome | V100-V143 | 21 |
| Firefox | V109-V145 | 13 |
| Safari | V15-V18 | 15 |
| Edge | V101-V142 | 10 |
| OkHttp | 3.x-5.x | 8 |

## Testing

```bash
cargo test                           # Run all tests
cargo test --ignored                 # Include network tests
cargo bench                          # Run benchmarks
```

## License

MIT License
