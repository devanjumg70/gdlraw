# Chromenet Limitations

This document describes known limitations, unsupported features, and scenarios where chromenet may not be suitable.

---

## Unsupported Protocols

### ❌ HTTP/3 (QUIC)
The `quic` module contains configuration types and connection stubs, but **no actual HTTP/3 implementation exists**.

```rust
// quic/config.rs exists with QuicConfig builder
// quic/connection.rs has placeholder types
// No quinn or other QUIC runtime integration
```

**Impact**: Cannot connect to HTTP/3-only servers or use QUIC transport.

**Workaround**: Servers typically fallback to HTTP/2; most use cases unaffected.

---

### ❌ Proxy Auto-Configuration (PAC)
No support for PAC/WPAD proxy discovery.

**Impact**: Enterprise networks using PAC files require manual proxy configuration.

**Workaround**: Extract proxy settings from browser or system and configure manually.

---

## Implementation Gaps

### ⚠️ CT Verification (Partial)
Certificate Transparency verification is **95% complete**:
- ✅ SCT list parsing (RFC 6962)
- ✅ Log registry and lookup
- ✅ Timestamp validation
- ✅ Requirements checking (`NotRequired`, `SoftFail`, `Required`)
- ⚠️ ECDSA signature verification uses placeholder (accepts any non-empty signature from known logs)

**Impact**: CT validation may pass when signature is invalid.

**Severity**: Low - infrastructure is complete; signature verification needs BoringSSL ECDSA integration.

---

### ⚠️ BoxedSocket Detection Limitation
Dead socket detection is **fully implemented** in `SocketType` (`client.rs`):
- ✅ `check_tcp_connected()` uses `peer_addr()` + `try_read()` to detect EOF/RST/FIN
- ✅ `check_ssl_connected()` checks underlying TCP stream
- ✅ `WrappedSocket::is_usable()` follows Chromium's `IdleSocket::IsUsable()` pattern

However, `BoxedSocket::is_connected()` returns `true` because trait objects can't easily call the inner method.

**Impact**: Pool cleanup uses `BoxedSocket`, so some stale sockets may persist until first I/O failure.

**Workaround**: Retry on connection errors (automatic with `HttpNetworkTransaction`).

---

## Platform Constraints

### Cookie Extraction
Browser cookie import has platform-specific limitations:

| Browser | Linux | macOS | Windows |
|---------|-------|-------|---------|
| Chrome | ✅ | ✅ | ✅ |
| Firefox | ✅ | ✅ | ✅ |
| Safari | ❌ | ✅ | ❌ |

Safari uses binary cookie format only available on macOS.

---

### TLS Library
**BoringSSL only** - by design, to match Chromium's TLS fingerprint.

❌ Not compatible with:
- `rustls`
- `native-tls`
- `openssl` (except for testing)

**Impact**: Cannot use system certificate stores via native-tls.

---

## API Constraints

### Global Static State
Some components use static singletons:

```rust
// urlrequest/request.rs
static POOL: LazyLock<...>
static FACTORY: LazyLock<...>
```

**Impact**: Cannot create fully isolated client contexts (for testing or multi-tenant).

**Workaround**: Use `Client::builder()` which manages its own pool.

---

### No Async Cookie Store
`CookieMonster` uses synchronous `DashMap` locking.

**Impact**: High cookie volume may cause contention.

---

## Not Recommended For

| Use Case | Reason | Alternative |
|----------|--------|-------------|
| HTTP/3 exclusive | No QUIC support | Use `h3` + `quinn` |
| System TLS integration | BoringSSL only | Use `reqwest` with native-tls |
| Minimal binary size | Heavy dependencies | Use `ureq` |
| Embedded/no_std | Requires tokio runtime | Use `embedded-nal` |
