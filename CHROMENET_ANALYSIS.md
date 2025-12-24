# chromenet: Comprehensive Codebase Analysis

> **Status**: Experimental (unpublished)  
> **Last Updated**: Dec 24, 2024  
> **Total Lines**: 3,263

---

## 1. What is chromenet?

**chromenet** is a browser-grade HTTP client library built by porting Chromium's C++ `net` stack to Rust. Unlike `reqwest` which abstracts away networking, chromenet exposes low-level control for:

- **Perfect browser impersonation** (TLS fingerprinting, HTTP/2 settings, header order)
- **Cookie management** mirroring RFC 6265 with Chrome's CookieMonster
- **Connection pooling** with Chromium's exact limits (6 per host, 256 total)
- **Proxy support** (HTTP CONNECT, SOCKS5, partial HTTPS)

### Why Port Chromium?

| Aspect | reqwest | chromenet |
|--------|---------|-----------|
| TLS fingerprint | Generic | Chrome-exact ✅ |
| HTTP/2 SETTINGS | Defaults | Configurable ✅ |
| Header order | Arbitrary | Preserved ✅ |
| Connection pool | Simple | Chromium-like ✅ |
| Cookie store | Optional | CookieMonster ✅ |
| Bot detection bypass | ❌ | Primary goal ✅ |

---

## 2. Current Architecture

```
chromenet/src/ (3,263 lines)
├── base/        (598 lines) - NetError (549), LoadState
├── cookies/     (646 lines) - CookieMonster, Persistence
├── http/        (563 lines) - Transaction, StreamFactory, Retry
├── socket/      (1,227 lines) - Pool, ConnectJob, TLS, Proxy
└── urlrequest/  (490 lines) - Context, Device, Request, Job
```

---

## 3. Module-by-Module Assessment

### 3.1 `socket/` - Connection Layer (1,227 lines)

| Component | Status | Notes |
|-----------|--------|-------|
| `pool.rs` (407) | ✅ Good | Chromium limits, request queuing, idle cleanup |
| `connectjob.rs` (382) | ⚠️ Partial | Happy Eyeballs ✅, SOCKS5 ✅, HTTPS proxy incomplete |
| `tls.rs` (106) | ✅ Good | BoringSSL config, Chrome fingerprint |
| `stream.rs` (79) | ✅ New | BoxedSocket for TLS-in-TLS foundation |
| `proxy.rs` (48) | ✅ Good | HTTP/SOCKS5 config |
| `client.rs` (105) | ⚠️ Redundant | SocketType enum + StreamSocket trait overlap |

**Issues:**
1. **HTTPS proxy incomplete** - `https_proxy_handshake()` doesn't do real TLS-in-TLS
2. **Dual socket abstractions** - `SocketType` enum vs `StreamSocket` trait

### 3.2 `http/` - HTTP Layer (563 lines)

| Component | Status | Notes |
|-----------|--------|-------|
| `transaction.rs` (209) | ✅ Good | State machine, retry integrated |
| `streamfactory.rs` (139) | ✅ Good | H1/H2 via ALPN |
| `orderedheaders.rs` (57) | ✅ Good | Insertion order preserved |
| `retry.rs` (154) | ✅ New | Exponential backoff, 8 RetryReasons |

**Issues:**
1. **No response body streaming** - Currently headers only
2. **No request body support** - Only empty bodies

### 3.3 `cookies/` - Cookie Management (646 lines)

| Component | Status | Notes |
|-----------|--------|-------|
| `monster.rs` (264) | ✅ Good | RFC 6265 domain/path matching, LRU eviction |
| `persistence.rs` (155) | ✅ New | JSON save/load |
| `canonical_cookie.rs` (69) | ✅ Good | Full cookie attributes |

**Issues:**
1. **No browser cookie extraction** - Only in-memory storage

### 3.4 `urlrequest/` - Public API (490 lines)

| Component | Status | Notes |
|-----------|--------|-------|
| `device.rs` (160) | ✅ Good | 78 Chrome DevTools devices, Client Hints |
| `context.rs` (120) | ✅ Good | URLRequestContext pattern |
| `job.rs` (138) | ✅ Good | URLRequestHttpJob |
| `request.rs` (51) | ⚠️ Minimal | Thin wrapper |

### 3.5 `base/` - Foundation (598 lines)

| Component | Status | Notes |
|-----------|--------|-------|
| `neterror.rs` (549) | ✅ Good | 100+ Chromium error codes |
| `loadstate.rs` (47) | ✅ Good | Progress reporting |

---

## 4. What's Working Well

1. **Connection pooling** - Chromium-accurate limits, priority queuing, idle cleanup
2. **Cookie management** - RFC 6265 compliant, per-domain + global limits
3. **TLS configuration** - Chrome fingerprint via BoringSSL
4. **Header ordering** - Preserved for fingerprinting
5. **Retry logic** - Chromium-like exponential backoff
6. **Device emulation** - 78 DevTools devices with Client Hints

---

## 5. Critical Gaps

### 5.1 Must Fix (Broken/Incomplete)

| Gap | Impact | Effort |
|-----|--------|--------|
| HTTPS proxy (TLS-in-TLS) | ❌ Won't work with HTTPS proxies | High |
| Response body streaming | ❌ Can't download files | Medium |
| Request body support | ❌ Can't POST/PUT data | Medium |

### 5.2 Should Add (Missing Features)

| Gap | Impact | Effort |
|-----|--------|--------|
| Browser cookie extraction | Major differentiation | High |
| HTTP/2 SETTINGS fingerprinting | Anti-bot bypass | Medium |
| Configurable timeouts | Production use | Low |
| Connection keep-alive tuning | Performance | Low |

### 5.3 Technical Debt

| Issue | Description | Fix |
|-------|-------------|-----|
| Dual socket types | `SocketType` enum + `StreamSocket` trait | Unify to StreamSocket |
| `anyhow` dependency | Unergonomic error handling | Remove, use `thiserror` only |
| `once_cell` dependency | Deprecated | Use `std::sync::OnceLock` |
| Unused variable warnings | 2 in connectjob.rs | Prefix with `_` |

---

## 6. Benchmark Performance

All 11 benchmarks pass ✅

| Category | Benchmark | Time | Rating |
|----------|-----------|------|--------|
| Headers | `headers_insert` | 122 ns | 🚀 Excellent |
| Headers | `headers_to_header_map` | 566 ns | ⚡ Fast |
| Pool | `pool_stats` | 66 ns | 🚀 Excellent |
| Pool | `pool_new` | 852 ns | ⚡ Fast |
| Pool | `pool_idle_socket_count` | 919 ns | ⚡ Fast |
| Device | `device_lookup` | 92 ns | 🚀 Excellent |
| Device | `set_device_overhead` | 917 ns | ⚡ Fast |
| Transaction | `transaction_new` | 600 ns | ⚡ Fast |
| Cookies | `cookie_parse_and_save` | 1.25 µs | ✅ Good |
| Cookies | `cookie_get_for_url` | 8.2 µs | ✅ Good |
| TLS | `tls_config_apply` | 3.7 ms | ⚠️ Expected |

**Verdict**: In-memory operations are sub-microsecond. Only crypto initialization is slow (expected).

---

## 7. Test Coverage

| Tests | Status |
|-------|--------|
| `test_backoff_exponential` | ✅ |
| `test_backoff_capped` | ✅ |
| `test_should_retry` | ✅ |
| `test_no_retry_config` | ✅ |
| `test_save_load_roundtrip` | ✅ |

**Gap**: No integration tests, no network tests.

---

## 8. Dependencies Assessment

| Dependency | Status | Notes |
|------------|--------|-------|
| `tokio` | ✅ Keep | Async runtime |
| `boring` | ✅ Keep | BoringSSL (Chrome TLS) |
| `hyper` | ✅ Keep | HTTP/1.1 + HTTP/2 parsing |
| `dashmap` | ✅ Keep | Concurrent hash maps |
| `thiserror` | ✅ Keep | Error derivation |
| `serde` | ✅ Keep | Cookie persistence |
| `anyhow` | ❌ Remove | Use thiserror + Result |
| `once_cell` | ❌ Remove | Use std::sync::OnceLock |
| `h2` | ⚠️ Review | May duplicate hyper's H2 |

---

## 9. Recommended Actions

### Phase 1: Stabilize Core (1-2 weeks)

1. **[ ] Fix response body streaming**
   - Add `ResponseBody` type with `chunk()`, `text()`, `json()` methods
   
2. **[ ] Add request body support**
   - Accept `impl Into<Body>` in transaction

3. **[ ] Unify socket types**
   - Remove `SocketType` enum, use `BoxedSocket` everywhere

4. **[ ] Remove anyhow/once_cell**
   - All functions return `Result<T, NetError>`

### Phase 2: Complete HTTPS Proxy (2-3 weeks)

5. **[ ] Refactor ConnectJob return type**
   - Return `BoxedSocket` instead of `SocketType`

6. **[ ] Implement TLS-in-TLS**
   - TLS to proxy → HTTP CONNECT → TLS to target

### Phase 3: Browser Impersonation (4-6 weeks)

7. **[ ] HTTP/2 SETTINGS fingerprinting**
   - Configurable SETTINGS frame

8. **[ ] Browser cookie extraction**
   - Chrome (encrypted), Firefox (plain), Safari (binary)

---

## 10. Suggested File Changes

```diff
# Remove
- src/socket/client.rs  (merge into stream.rs)

# Refactor
~ src/socket/connectjob.rs  (return BoxedSocket)
~ src/http/transaction.rs   (add body support)

# Add
+ src/http/body.rs          (ResponseBody, RequestBody)
+ src/impersonate/mod.rs    (TLS + HTTP/2 fingerprints)
+ src/browser_cookies/mod.rs (Chrome/Firefox/Safari extraction)
```

---

## 11. Summary

| Metric | Value |
|--------|-------|
| **Lines of Code** | 3,263 |
| **Modules** | 5 |
| **Tests Passing** | 5/5 |
| **Benchmarks** | 11 (all pass) |
| **Core Completion** | ~65% |
| **Production Ready** | No (missing body support, HTTPS proxy) |

**Ground Reality**: Solid foundation, but 2-3 weeks from MVP usability.

**Unique Value Proposition**: Once complete, chromenet will be the only Rust crate offering:
- Chrome-accurate TLS fingerprinting
- Preserved header ordering
- CookieMonster-compatible storage
- Browser cookie extraction

This fills a gap that `reqwest`, `ureq`, and `surf` don't address.
