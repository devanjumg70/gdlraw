# Performance Findings

## Benchmark Results

| Benchmark | Time | Notes |
|-----------|------|-------|
| **tls_config_apply** | 3.63 ms | ⚠️ Slowest - BoringSSL init |
| cookie_get_for_url | 14.0 µs | Moderate |
| set_device_overhead | 853 ns | Acceptable |
| headers_to_header_map | 593 ns | Good |
| transaction_new | 533 ns | Good |
| cookie_parse_and_save | 310 ns | Good |
| pool_contention | 176 ns | Excellent |
| headers_insert | 117 ns | Excellent |
| device_lookup | 92 ns | Excellent |

---

## Hotspots

### 1. TLS Configuration (3.6ms)

**Location**: [tls.rs:35-59](file:///home/ubuntu/projects/chromium/dl/chromenet/src/socket/tls.rs#L35-L59)

Each connection calls `SslConnectorBuilder::new()`. BoringSSL initialization is expensive.

**Recommendation**: Cache `SslConnector` per TLS config and reuse.

```diff
- let mut builder = SslConnector::builder(SslMethod::tls())?;
- tls_config.apply_to_builder(&mut builder)?;
- let connector = builder.build();
+ let connector = get_cached_connector(&tls_config);
```

---

### 2. Cookie URL Lookup (14µs)

**Location**: [monster.rs:37-59](file:///home/ubuntu/projects/chromium/dl/chromenet/src/cookies/monster.rs#L37-L59)

Iterates all cookies for domain, clones each match.

**Recommendations**:
1. Use trie-based domain index
2. Return `Arc<CanonicalCookie>` instead of cloning

---

### 3. No Connection Timing

No benchmarks measure actual connection establishment:
- DNS resolution time
- TCP handshake latency
- TLS handshake latency
- Proxy CONNECT time

**Recommendation**: Add integration benchmarks with mock servers.

---

## Missing Benchmarks

| Component | Why |
|-----------|-----|
| Happy Eyeballs | Not implemented |
| Socket Pool exhaustion | Returns error instead of queuing |
| H2 multiplex | Requires live connection |
| Redirect chain | End-to-end test |
