# HTTP Module

The HTTP module handles HTTP transactions, caching, streaming, and multipart uploads.

## Key Types

### HttpNetworkTransaction
The core transaction driver that manages HTTP request/response lifecycle.

```rust
let mut tx = HttpNetworkTransaction::new(url, method, factory, cookies);
tx.set_body("POST data");
tx.add_header("Content-Type", "application/json")?;
tx.start().await?;
```

### HttpCache
In-memory HTTP cache with RFC 7234 compliance.

```rust
let cache = HttpCache::new();
cache.store(&url, "GET", &response, body);

if let Some(entry) = cache.get(&url, "GET") {
    // Use cached response
}
```

**Features**:
- Cache-Control parsing (max-age, no-store, no-cache)
- ETag/If-None-Match conditional requests
- Last-Modified/If-Modified-Since support
- LRU eviction with size limits
- Thread-safe via DashMap

### ResponseBody Streaming
Memory-efficient streaming for large responses.

```rust
use futures::StreamExt;

let stream = body.into_stream();
while let Some(chunk) = stream.next().await {
    let bytes = chunk?;
    // Process chunk
}
```

### Multipart Forms
RFC 2046 multipart/form-data encoding.

```rust
use chromenet::http::multipart::{Form, Part};

let form = Form::new()
    .text("field", "value")
    .part("file", Part::bytes(data)
        .file_name("doc.pdf")
        .content_type("application/pdf"));

let body = form.into_body();
```

## Files

| File | Purpose |
|------|---------|
| `transaction.rs` | HttpNetworkTransaction state machine |
| `httpcache.rs` | HTTP cache with Cache-Control |
| `multipart.rs` | Form uploads |
| `responsebody.rs` | Body streaming |
| `requestbody.rs` | Request body handling |
| `streamfactory.rs` | H1/H2 stream creation |
| `orderedheaders.rs` | Header ordering for fingerprinting |
| `h2fingerprint.rs` | HTTP/2 fingerprinting |
| `digestauth.rs` | HTTP Digest authentication (RFC 7616) |
| `retry.rs` | Request retry logic |

---

## HTTP Digest Authentication

Full RFC 7616 implementation for Digest access authentication.

```rust
use chromenet::http::digestauth::DigestAuthHandler;

// Parse challenge from WWW-Authenticate header
let handler = DigestAuthHandler::parse_challenge(
    r#"realm="test", nonce="abc123", qop="auth", algorithm=SHA-256"#
)?;

// Generate Authorization header
let auth_token = handler.generate_auth_token(
    "GET",
    "/protected",
    "username",
    "password"
);
```

**Supported Algorithms**: MD5, MD5-sess, SHA-256, SHA-256-sess
**QoP Modes**: auth, auth-int
