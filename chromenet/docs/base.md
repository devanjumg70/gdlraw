# Base Module

## Files
- [neterror.rs](file:///home/ubuntu/projects/gdlraw/chromenet/src/base/neterror.rs) - Error codes
- [loadstate.rs](file:///home/ubuntu/projects/gdlraw/chromenet/src/base/loadstate.rs) - Request states
- [context.rs](file:///home/ubuntu/projects/gdlraw/chromenet/src/base/context.rs) - Error context helpers

> [!TIP]
> See [errors.md](errors.md) for comprehensive error handling documentation.

---

## NetError

Comprehensive error enum mapping Chromium's `net/base/net_error_list.h`. Contains 120+ variants.

### Categories

| Range | Category | Examples |
|-------|----------|----------|
| -100s | Connection | `ConnectionClosed`, `NameNotResolved` |
| -100s | SSL/TLS | `SslProtocolError`, `EchNotNegotiated` |
| -300s | HTTP | `InvalidUrl`, `TooManyRedirects` |
| -10020s | Cookie | `BrowserNotFound`, `CookieDecryptionFailed` |

### Context-Rich Variants

```rust
NetError::ConnectionFailedTo { host, port, source }
NetError::NameNotResolvedFor { domain, source }
NetError::SslHandshakeFailedWith { host, reason }
```

### From Implementations

| From | To |
|------|----|
| `std::io::Error` | Mapped by `ErrorKind` |
| `url::ParseError` | `NetError::InvalidUrl` |
| `rusqlite::Error` | Cookie database errors |

---

## IoResultExt

Extension trait for ergonomic error context:

```rust
use chromenet::base::context::IoResultExt;

stream.connection_context("example.com", 443)?;
// Error: "Connection to example.com:443 failed: ..."
```

---

## LoadState

Enum representing request progress states. Matches `net/base/load_states.h`.

```rust
pub enum LoadState {
    Idle,
    WaitingForStalledSocketPool,
    WaitingForAvailableSocket,
    ResolvingHost,
    Connecting,
    SslHandshake,
    SendingRequest,
    WaitingForResponse,
    ReadingResponse,
    // ... and more
}
```

