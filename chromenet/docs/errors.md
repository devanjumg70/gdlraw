# Error Handling

`chromenet` uses structured error handling based on `thiserror`, avoiding opaque types like `anyhow` to give callers full control over error matching.

## NetError

The primary error type mirrors Chromium's `net/base/net_error_list.h`.

### Error Code Ranges

| Range | Category | Examples |
|-------|----------|----------|
| -100s | Connection | `ConnectionClosed`, `ConnectionRefused`, `NameNotResolved` |
| -200s | Certificates | `CertDateInvalid`, `CertAuthorityInvalid` |
| -300s | HTTP | `InvalidUrl`, `TooManyRedirects`, `EmptyResponse` |
| -300s | HTTP/2 | `Http2ProtocolError`, `Http2FlowControlError` |
| -10020s | Cookie | `BrowserNotFound`, `CookieDecryptionFailed` |

### Chromium Alignment

| Chromium | chromenet | Notes |
|----------|-----------|-------|
| `net_error_list.h` | `neterror.rs` | 110+ variants |
| NetLog for context | `#[source]` chaining | We embed context in errors |
| Macro-generated codes | `as_i32()` method | FFI compatible |

---

## Context-Rich Errors

Some variants include contextual data for better debugging:

```rust
#[error("Connection to {host}:{port} failed")]
ConnectionFailedTo {
    host: String,
    port: u16,
    #[source]
    source: Arc<io::Error>,
}

#[error("DNS resolution for {domain} failed")]
NameNotResolvedFor {
    domain: String,
    #[source]
    source: Arc<io::Error>,
}
```

---

## Extension Trait: `IoResultExt`

For ergonomic error context, use the extension trait:

```rust
use chromenet::base::context::IoResultExt;

// Add DNS context
let addrs = tokio::net::lookup_host(addr)
    .await
    .dns_context("example.com")?;
// Error: "DNS resolution for example.com failed: ..."

// Add connection context
let stream = TcpStream::connect(addr)
    .await
    .connection_context("example.com", 443)?;
// Error: "Connection to example.com:443 failed: ..."
```

### Available Methods

| Method | Creates |
|--------|---------|
| `.connection_context(host, port)` | `NetError::ConnectionFailedTo` |
| `.dns_context(domain)` | `NetError::NameNotResolvedFor` |

---

## `From` Implementations

Automatic conversions:

| From | To | Mapping |
|------|----|---------|
| `std::io::Error` | `NetError` | Based on `ErrorKind` |
| `url::ParseError` | `NetError::InvalidUrl` | Always |
| `rusqlite::Error` | `NetError` | `CookieDatabaseLocked` or `CookieDatabaseError` |

---

## Cookie Errors

Previously `CookieExtractionError`, now unified into `NetError`:

| Old | New |
|-----|-----|
| `CookieExtractionError::BrowserNotFound` | `NetError::BrowserNotFound { browser }` |
| `CookieExtractionError::DecryptionFailed` | `NetError::CookieDecryptionFailed { browser, reason }` |
| `CookieExtractionError::DatabaseLocked` | `NetError::CookieDatabaseLocked` |

> [!WARNING]
> `CookieExtractionError` is deprecated. Use `NetError` directly.

---

## Why Not `anyhow`?

`chromenet` is a **library**, not an application:

| | `anyhow` | `thiserror` |
|-|----------|-------------|
| Best for | Applications | Libraries |
| Type safety | Erased | Preserved |
| Caller matching | Downcast required | Direct match |
| Context | String only | Typed fields |

> "Use thiserror if you are a library that wants to design your own dedicated error type(s)"
> — dtolnay (anyhow author)
