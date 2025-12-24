# Base Module

## Files
- [neterror.rs](file:///home/ubuntu/projects/chromium/dl/chromenet/src/base/neterror.rs) (546 lines)
- [loadstate.rs](file:///home/ubuntu/projects/chromium/dl/chromenet/src/base/loadstate.rs) (48 lines)

---

## NetError

Comprehensive error enum mapping Chromium's `net/base/net_error_list.h`. Contains 120+ variants.

### Categories

| Range | Category | Examples |
|-------|----------|----------|
| -100s | Connection | `ConnectionClosed`, `ConnectionReset`, `NameNotResolved` |
| -100s | SSL/TLS | `SslProtocolError`, `SslVersionOrCipherMismatch`, `EchNotNegotiated` |
| -300s | HTTP | `InvalidUrl`, `TooManyRedirects`, `EmptyResponse` |
| -300s | HTTP/2 | `Http2ProtocolError`, `Http2FlowControlError` |

### Key Methods
```rust
impl NetError {
    pub fn as_i32(&self) -> i32;  // Convert to Chromium error code
}

impl From<i32> for NetError {
    fn from(code: i32) -> Self;  // Parse from Chromium code
}
```

---

## LoadState

Enum representing request progress states. Matches `net/base/load_states.h`.

```rust
pub enum LoadState {
    Idle,
    WaitingForStalledSocketPool,
    WaitingForAvailableSocket,
    WaitingForDelegate,
    WaitingForCache,
    WaitingForAppCache,
    DownloadingPacFile,
    ResolvingProxyForUrl,
    ResolvingHost,
    Connecting,
    SslHandshake,
    SendingRequest,
    WaitingForResponse,
    ReadingResponse,
}
```

> [!NOTE]
> `LoadState` is defined but **not currently used** in the codebase. The transaction state machine uses a private enum.
