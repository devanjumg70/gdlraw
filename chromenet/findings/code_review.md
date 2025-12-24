# Code Review Findings

## Summary
| Severity | Count |
|----------|-------|
| Critical | 3 |
| High | 4 |
| Medium | 5 |
| Low | 4 |

---

## Critical Issues

### 1. Socket Pool Fails Instead of Queuing
**File**: [pool.rs:88-90](file:///home/ubuntu/projects/chromium/dl/chromenet/src/socket/pool.rs#L88-L90)

```rust
if active_count + idle_count >= self.max_sockets_per_group {
    return Err(NetError::PreconnectMaxSocketLimit);  // ← Fails immediately
}
```

**Problem**: When socket limit is reached, requests fail immediately. Chromium queues requests until a socket becomes available.

**Impact**: Under load, requests will fail instead of waiting.

**Fix**: Implement request queuing with `tokio::sync::watch` or channel.

---

### 2. `is_connected()` Always Returns True
**File**: [client.rs:62-65](file:///home/ubuntu/projects/chromium/dl/chromenet/src/socket/client.rs#L62-L65)

```rust
impl StreamSocket for SocketType {
    fn is_connected(&self) -> bool {
        true  // ← Always returns true!
    }
}
```

**Problem**: Dead/closed sockets are returned from pool as "valid".

**Impact**: Requests on dead sockets fail with confusing errors.

**Fix**: Use `poll_peek` or detect `ECONNRESET` on first I/O.

---

### 3. No Happy Eyeballs (IPv4/IPv6 Racing)
**File**: [connectjob.rs:38-43](file:///home/ubuntu/projects/chromium/dl/chromenet/src/socket/connectjob.rs#L38-L43)

```rust
for addr in addrs {
    if let Ok(s) = TcpStream::connect(addr).await {
        stream = Some(s);
        break;  // ← Sequential, not parallel
    }
}
```

**Problem**: Tries addresses sequentially. If first address is slow/unreachable, connection is delayed.

**Impact**: Poor performance on dual-stack hosts.

**Fix**: Use `tokio::select!` with 250ms delay between IPv6 and IPv4 attempts.

---

## High Priority

### 4. Proxy CONNECT Response Not Fully Parsed
**File**: [connectjob.rs:74-76](file:///home/ubuntu/projects/chromium/dl/chromenet/src/socket/connectjob.rs#L74-L76)

```rust
let mut buf = [0u8; 1024];
let n = stream.read(&mut buf).await...
```

**Problem**: Reads only 1024 bytes, doesn't handle response spanning multiple reads or check for complete headers.

---

### 5. Cookie Domain Matching Incomplete
**File**: [monster.rs:42](file:///home/ubuntu/projects/chromium/dl/chromenet/src/cookies/monster.rs#L42)

```rust
if let Some(entry) = self.store.get(host) {  // ← Exact match only
```

**Problem**: Cookies for `.example.com` won't match `sub.example.com`.

---

### 6. LoadState Enum Unused
**File**: [loadstate.rs](file:///home/ubuntu/projects/chromium/dl/chromenet/src/base/loadstate.rs)

Defined but never used. `HttpNetworkTransaction` uses private `State` enum.

---

### 7. SOCKS5/HTTPS Proxy Not Implemented
**File**: [connectjob.rs:87-91](file:///home/ubuntu/projects/chromium/dl/chromenet/src/socket/connectjob.rs#L87-L91)

```rust
_ => {
    eprintln!("Unsupported proxy type");
    return Err(NetError::ConnectionFailed);
}
```

---

## Medium Priority

### 8. Active Count Race Condition
**File**: [pool.rs:102-103](file:///home/ubuntu/projects/chromium/dl/chromenet/src/socket/pool.rs#L102-L103)

```rust
self.active_per_group.entry(group_id.clone()).and_modify(|c| *c += 1);
self.total_active.fetch_add(1, Ordering::Relaxed);
```

**Problem**: Non-atomic increment between group and total counts.

---

### 9. No Connection Timeout
**File**: [connectjob.rs](file:///home/ubuntu/projects/chromium/dl/chromenet/src/socket/connectjob.rs)

No timeout on TCP connect or TLS handshake.

---

### 10. Device User-Agent Version Placeholder
**File**: [device.rs:66](file:///home/ubuntu/projects/chromium/dl/chromenet/src/urlrequest/device.rs#L66)

```rust
user_agent: "...Chrome/%s Mobile..."  // ← Not substituted
```

---

### 11. OrderedHeaderMap Error Swallowed
**File**: [transaction.rs:65](file:///home/ubuntu/projects/chromium/dl/chromenet/src/http/transaction.rs#L65)

```rust
let _ = self.request_headers.insert(key, value);  // Error ignored
```

---

### 12. TLS GREASE Not Explicitly Enabled
**File**: [tls.rs:52-53](file:///home/ubuntu/projects/chromium/dl/chromenet/src/socket/tls.rs#L52-L53)

```rust
// GREASE - Note: BoringSSL enables GREASE by default in SSL_CTX_new?
```

---

## Low Priority

### 13. Global Static Context
**File**: [request.rs:13-16](file:///home/ubuntu/projects/chromium/dl/chromenet/src/urlrequest/request.rs#L13-L16)

Static `POOL` and `FACTORY` prevent multiple isolated contexts.

### 14. No Cookie Eviction
**File**: [monster.rs](file:///home/ubuntu/projects/chromium/dl/chromenet/src/cookies/monster.rs)

No limit on stored cookies per domain or total.

### 15. Idle Socket Cleanup
**File**: [pool.rs](file:///home/ubuntu/projects/chromium/dl/chromenet/src/socket/pool.rs)

No background task to prune stale idle sockets.

### 16. Missing `Content-Length` Check
**File**: [transaction.rs](file:///home/ubuntu/projects/chromium/dl/chromenet/src/http/transaction.rs)

No validation of response body length.
