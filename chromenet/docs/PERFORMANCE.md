# Performance Best Practices

chromenet is designed for high-performance, browser-like networking. To achieve optimal performance, follow these guidelines.

## 1. Context Reuse (CRITICAL)

**Impact:** Reuse saves **3.7ms** TLS setup overhead per connection.
**Why:** Creating a `URLRequestContext` initializes the SSL connector, which is expensive.

✅ **DO:** Reuse `URLRequestContext` across requests (or use `URLRequest::new` which uses the global singleton).
```rust
// Recommended: Use the global singleton (implicitly reuses context)
for url in urls {
    let req = URLRequest::new(url)?;
    req.start().await?;
}

// Advanced: Explicit context reuse
let ctx = URLRequestContext::new(); 
for url in urls {
    let req = URLRequest::with_context(&ctx, url)?;
    req.start().await?;
}
```

❌ **DON'T:** Create a new context for every request.
```rust
for url in urls {
    let ctx = URLRequestContext::new(); // ⚠️ Wastes 3.7ms per iteration
    let req = URLRequest::with_context(&ctx, url)?;
}
```

## 2. Connection Pooling

**Impact:** **6.4x faster** requests (1.2s → 185ms).
**Why:** Reusing an established TCP/TLS connection skips DNS resolution (10-50ms), TCP handshake (1 RTT), and TLS handshake (2 RTTs).

- **Limit:** Max 6 connections per host (Chromium default).
- **Limit:** Max 256 total connections.
- **Behavior:** Connections are kept alive for 30s.

## 3. HTTP/2 Multiplexing

**Impact:** Zero additional TLS overhead for concurrent requests.
**Why:** H2 allows multiple streams over a single TCP/TLS connection.

- **Behavior:** Requests to the same H2 server automatically share the underlying connection.
- **Efficiency:** 10 requests to `https://example.com` = 1 TCP connection.

## 4. Cookie Management

**Impact:** Prevents unbounded memory growth and ensures fast lookups.

- **Limit:** 50 cookies per domain (LRU eviction).
- **Limit:** 3000 cookies total.
- **Validation:** Public Suffix List (PSL) checks are cached for performance (50ns vs 500ns).

## 5. DNS Caching

**Impact:** saves 10-50ms per repeated domain.

- **Implementation:** Uses `hickory-resolver`'s internal caching.
- **Performance:**
    - First lookup: Network latency (~2.5ms - 50ms)
    - Cached lookup: ~280μs (8x faster)

## Performance Metrics (Reference)

| Operation | Latency | Note |
|-----------|---------|------|
| Cold Request (DNS+TCP+TLS) | 1.18s | Baseline |
| Warm Request (Pooled) | 185ms | **6.4x Faster** |
| DNS Lookup (Cold) | 2.5ms | Network dependent |
| DNS Lookup (Cached) | 0.28ms | **8x Faster** |
| PSL Check (Cold) | 500ns | Allocation heavy |
| PSL Check (Cached) | 50ns | **10x Faster** |
