# Chromenet & Wreq Alignment Analysis

This document analyzes how well the master plan aligns with Chromium's approach and wreq's implementation, identifying issues and areas for improvement.

---

## Table of Contents

1. [Architectural Alignment](#1-architectural-alignment)
2. [TLS & Emulation Gap Analysis](#2-tls--emulation-gap-analysis)
3. [Proxy Implementation Gap](#3-proxy-implementation-gap)
4. [Cookie System Comparison](#4-cookie-system-comparison)
5. [Client API Gap](#5-client-api-gap)
6. [Critical Issues to Fix](#6-critical-issues-to-fix)
7. [Recommendations](#7-recommendations)

---

## 1. Architectural Alignment

### Chromium Mapping (Excellent ✅)

chromenet correctly mirrors Chromium's architecture:

| Chromium (C++) | chromenet (Rust) | Status |
|----------------|------------------|--------|
| `net::ClientSocketPool` | `socket/pool.rs` | ✅ Implemented (6 conn/host, 256 total) |
| `net::HttpNetworkTransaction` | `http/transaction.rs` | ✅ State machine with retry logic |
| `net::ConnectJob` | `socket/connectjob.rs` | ✅ DNS → TCP → SSL pipeline |
| `net::CookieMonster` | `cookies/monster.rs` | ✅ Full implementation with LRU eviction |
| `net::URLRequest` | `urlrequest/request.rs` | ⚠️ Basic facade only |
| `os_crypt::OSCrypt` | `cookies/oscrypt.rs` | ✅ Cross-platform decryption |

### Wreq Philosophy (Partial Match ⚠️)

wreq prioritizes **developer ergonomics** with:
- Fluent builder APIs (`Client::new().get(url).send().await`)
- Shortcut functions (`wreq::get()`, `wreq::post()`)
- Trait-based extensibility (`CookieStore`, `IntoCookieStore`)

**Gap**: chromenet lacks high-level wrapper API; currently exposes internal machinery directly.

---

## 2. TLS & Emulation Gap Analysis

### Current State

| Feature | chromenet | wreq/wreq-util | Gap |
|---------|-----------|----------------|-----|
| Impersonation targets | **8** (Chrome124/128, FF128/129, Safari17/18, OkHttp4/5) | **70+** (Chrome 100-143, Edge, Opera, Safari iOS/iPad, Firefox variants) | **Large** |
| TLS Options | ✅ Comprehensive builder | ✅ Comprehensive builder | Aligned |
| ALPS support | ⚠️ Missing | ✅ Full (alps_protocols, alps_use_new_codepoint) | Gap |
| Extension permutation | ✅ Implemented | ✅ Implemented | Aligned |
| ECH GREASE | ✅ Implemented | ✅ Implemented | Aligned |
| Certificate compression | ✅ BROTLI, ZLIB, ZSTD | ✅ Same | Aligned |

### wreq-util Emulation Profiles (Missing from chromenet)

```
Chrome: 100-143 (37 versions)
Edge: 101, 122, 127, 131, 134-142 (12 versions)
Opera: 116-119 (4 versions)
Safari: 15.3-26.2, iOS variants, iPad variants (27 versions)
Firefox: 109, 117, 128, 133-145, Private/Android variants (14 versions)
OkHttp: 3.9-5.0 (8 versions)
```

### Issue: Emulation OS Pattern Missing

wreq-util has `EmulationOS` enum (Windows, MacOS, Linux, Android, iOS) that affects:
- Platform client hints
- User-Agent suffix
- Mobile detection

chromenet has `DeviceProfile` in `urlrequest/device.rs` but doesn't integrate with TLS emulation.

---

## 3. Proxy Implementation Gap

### Critical Gap: chromenet lacks sophisticated proxy matching

| Feature | chromenet | wreq | Status |
|---------|-----------|------|--------|
| Basic proxy configuration | ✅ `socket/proxy.rs` | ✅ | Aligned |
| Environment variable parsing | ❌ Missing | ✅ `HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY`, `NO_PROXY` | **Gap** |
| NO_PROXY domain matching | ❌ Missing | ✅ Full curl-compatible rules | **Gap** |
| IP/CIDR matching | ❌ Missing | ✅ `IpMatcher` with CIDR support | **Gap** |
| System proxy detection (macOS) | ❌ Missing | ✅ `proxy/mac.rs` | **Gap** |
| System proxy detection (Windows) | ❌ Missing | ✅ `proxy/win.rs` | **Gap** |
| Unix socket proxy | ❌ Missing | ✅ `proxy/uds.rs` | **Gap** |
| Proxy auth (basic) | ✅ `socket/authcache.rs` | ✅ | Aligned |
| Custom proxy headers | ❌ Missing | ✅ `Intercept.custom_headers()` | **Gap** |

### Wreq Matcher Features to Port

```rust
// wreq/src/proxy/matcher.rs - Key features:
- Matcher::from_system()      // Auto-detect OS proxies
- Matcher::from_env()         // Parse env vars
- Builder::all(), .http(), .https()  // Per-scheme proxies
- NoProxy::from_string()      // Parse NO_PROXY rules
- DomainMatcher with suffix matching
- IpMatcher with CIDR support
```

---

## 4. Cookie System Comparison

### Architecture Comparison

| Aspect | chromenet | wreq | Notes |
|--------|-----------|------|-------|
| Core store | `CookieMonster` (Chromium-style) | `cookie::CookieJar` wrapper | chromenet is more robust |
| Thread safety | `DashMap` based | `RwLock<CookieJar>` | Both safe |
| Browser extraction | ✅ Chrome, Firefox, Safari, Edge, Brave, Opera | ❌ Not supported | chromenet advantage |
| Decryption | ✅ v10, v11, Keychain, DPAPI | ❌ N/A | chromenet advantage |
| PSL validation | ✅ `psl.rs` | ❌ Not implemented | chromenet advantage |
| Persistence | ✅ `persistence.rs` | ❌ Not built-in | chromenet advantage |
| Import/Export | ✅ Netscape format, browser import | ❌ N/A | chromenet advantage |
| Trait-based API | ❌ Concrete type | ✅ `CookieStore` trait | wreq advantage |

### Recommendation

chromenet's CookieMonster is **superior** (Chromium-ported with full security features). Consider adding a `CookieStore` trait wrapper for wreq-style APIs.

---

## 5. Client API Gap

### Current API Comparison

**chromenet (Low-level)**:
```rust
use chromenet::urlrequest::URLRequest;
let response = URLRequest::new("https://example.com")
    .with_cookies(cookies)
    .send()
    .await?;
```

**wreq (High-level)**:
```rust
use wreq::{Client, Emulation};
let client = Client::builder()
    .emulation(Emulation::Chrome136)
    .cookie_store(true)
    .build()?;
let resp = client.get("https://example.com")
    .send().await?;
println!("{}", resp.text().await?);
```

### Missing in chromenet

1. **Client struct** wrapping URLRequestContext
2. **RequestBuilder** (method chaining for headers, body, etc.)
3. **Response helpers** (`.text()`, `.json()`, `.bytes()`)
4. **Shortcut functions** (`get()`, `post()`, `put()`, `delete()`)
5. **Multipart form data** (`multipart::Form`)
6. **WebSocket upgrade** (`client.ws()`)
7. **Middleware/Layer system** (interceptors)

---

## 6. Critical Issues to Fix

### High Priority

| Issue | Location | Fix |
|-------|----------|-----|
| CT Verification incomplete | `tls/ctverifier.rs:159` | Implement ECDSA signature verification |
| Debug logging in production | `connectjob.rs`, `transaction.rs`, `streamfactory.rs` | Replace `eprintln!` with `tracing` |
| No HTTP cache | Missing | Implement `HttpCache` per Chromium's design |
| No streaming body | `http/body.rs` | Only Empty body supported |
| HSTS not persisted | `tls/hsts.rs` | Memory only, add `TransportSecurityPersister` |

### Medium Priority

| Issue | Location | Fix |
|-------|----------|-----|
| No QUIC/HTTP3 | Missing | Long-term goal |
| No PAC file support | Missing | Port `net/proxy_resolution/` logic |
| Limited error context | `pool.rs:221`, `streamfactory.rs` | Add URL/host context to errors |

### Code Quality

| Issue | Fix |
|-------|-----|
| Unwraps in production | Review `digestauth.rs:311`, `monster.rs:263` |
| Missing `#[must_use]` | Add to `request_socket()`, `create_stream()` |

---

## 7. Recommendations

### Immediate Actions (Phase 5 Priority)

1. **Create `client.rs`** module with:
   - `Client` struct wrapping `URLRequestContext`
   - `ClientBuilder` for configuration
   - Connection pooling integration

2. **Create `request.rs`** builder:
   - Method chaining: `.header()`, `.body()`, `.timeout()`
   - Integration with `ImpersonateTarget`

3. **Create `response.rs`** helpers:
   - `.text()` → String
   - `.json::<T>()` → Deserialize
   - `.bytes()` → Bytes

### Emulation Expansion

Port wreq-util's 70+ emulation profiles to chromenet's `ImpersonateTarget`. Consider:
- Group by browser family (Chrome, Firefox, Safari, Edge)
- Use sub-enums or feature flags for all profiles
- Match wreq-util's `EmulationOS` pattern

### Proxy System

Port wreq's matcher.rs to `chromenet/src/proxy/`:
- `matcher.rs` → Main matcher logic
- `noproxy.rs` → NO_PROXY parsing
- `system.rs` → OS-specific detection (cfg-gated)

### Trait Unification

Create bridge traits for interoperability:
```rust
pub trait CookieStore: Send + Sync {
    fn set_cookies(&self, headers: &[HeaderValue], uri: &Uri);
    fn cookies(&self, uri: &Uri) -> Option<HeaderValue>;
}

impl CookieStore for CookieMonster { ... }
```

---

## Summary

| Area | Alignment | Action Required |
|------|-----------|-----------------|
| Core Architecture | ✅ Excellent | Maintain Chromium mapping |
| TLS Options | ✅ Good | Add ALPS support |
| Emulation Profiles | ⚠️ Limited | Expand from 8 to 70+ targets |
| Proxy System | ❌ Poor | Port wreq's matcher.rs |
| Cookie Management | ✅ Excellent | Add trait wrapper |
| Client API | ❌ Missing | Priority: Create high-level API |
| Advanced Features | ❌ Missing | Port multipart, WS, redirects |

**Bottom Line**: The foundation is solid (Chromium architecture), but the user-facing API layer is incomplete. Phase 5 (Client API) should be the immediate priority.
