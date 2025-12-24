# HTTP Module

## Files
- [transaction.rs](file:///home/ubuntu/projects/chromium/dl/chromenet/src/http/transaction.rs) (~210 lines)
- [streamfactory.rs](file:///home/ubuntu/projects/chromium/dl/chromenet/src/http/streamfactory.rs) (~140 lines)
- [orderedheaders.rs](file:///home/ubuntu/projects/chromium/dl/chromenet/src/http/orderedheaders.rs) (~60 lines)
- [retry.rs](file:///home/ubuntu/projects/chromium/dl/chromenet/src/http/retry.rs) (~155 lines)

---

## HttpNetworkTransaction

State machine for HTTP request/response lifecycle.

```mermaid
stateDiagram-v2
    [*] --> CreateStream
    CreateStream --> SendRequest
    SendRequest --> ReadHeaders : Success
    SendRequest --> CreateStream : Retry (reused socket)
    ReadHeaders --> Done
    Done --> [*]
```

### Features
- **Automatic retry** with exponential backoff (max 3 attempts)
- Auto-retry on reused socket failure
- Cookie storage from `Set-Cookie` headers
- H1/H2 protocol selection via ALPN

---

## Retry Module

Exponential backoff retry logic based on Chromium's `HttpNetworkTransaction`.

### RetryReason (retryable errors)
| Error | Description |
|-------|-------------|
| ConnectionReset | TCP RST received |
| ConnectionClosed | TCP FIN received |
| EmptyResponse | No data from server |
| SocketNotConnected | Socket disconnected |
| HttpRequestTimeout | Request timed out |

### RetryConfig
```rust
RetryConfig {
    max_attempts: 3,      // Chromium default
    base_delay_ms: 100,   // 100ms initial backoff
    max_delay_ms: 5000,   // Cap at 5 seconds
    jitter_factor: 0.1,   // ±10% jitter
}
```

## HttpStreamFactory

Creates HTTP streams from pooled sockets.

### Protocol Detection
```rust
let is_h2 = matches!(ssl_stream.ssl().selected_alpn_protocol(), Some(b"h2"));
```

### Stream Types
```rust
enum HttpStreamInner {
    H1(http1::SendRequest<...>),
    H2(http2::SendRequest<...>),
}
```

---

## OrderedHeaderMap

Header map preserving insertion order for fingerprinting.

```rust
pub struct OrderedHeaderMap {
    headers: Vec<(HeaderName, HeaderValue)>,
}
```

### Methods
| Method | Behavior |
|--------|----------|
| `insert` | Update in-place or append |
| `remove` | Filter by name |
| `get` | First match |
| `to_header_map` | Convert to `http::HeaderMap` |

> [!TIP]
> Header names are automatically lowercased by `HeaderName::from_str()`.
