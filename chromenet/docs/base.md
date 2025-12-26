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
| -10000s | Custom | `RedirectCycleDetected` (Moved from -900s to avoid Blob collision) |

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
    ObsoleteWaitingForAppCache, // Deprecated in Chromium
    DownloadingPacFile,
    ResolvingProxyForUrl,
    ResolvingHostInPacFile,     // NEW: Added for completeness
    ResolvingHost,
    Connecting,
    SslHandshake,
    EstablishingProxyTunnel,    // NEW: Added for completeness
    SendingRequest,
    WaitingForResponse,
    ReadingResponse,
}
```

> [!NOTE]
> `LoadState` matches Chromium's definitions, including deprecated states marked as obsolete.
