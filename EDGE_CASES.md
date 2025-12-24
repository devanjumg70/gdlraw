# Edge Case Implementation Plan

> Based on Chromium source analysis against chromenet codebase

---

## Priority 1: Critical (Will Cause Failures)

### 1.1 Dead Socket Detection on Reuse

**Chromium Pattern** (`transport_client_socket_pool.cc:785-805`):
```cpp
bool IdleSocket::IsUsable(const char** net_log_reason_utf8) const {
  if (socket->WasEverUsed()) {
    if (!socket->IsConnectedAndIdle()) {
      if (!socket->IsConnected()) {
        *net_log_reason_utf8 = kRemoteSideClosedConnection;
      } else {
        *net_log_reason_utf8 = kDataReceivedUnexpectedly;
      }
      return false;
    }
  }
  // Never-used sockets only need IsConnected check
  if (!socket->IsConnected()) {
    *net_log_reason_utf8 = kRemoteSideClosedConnection;
    return false;
  }
  return true;
}
```

**Current chromenet**: Only checks `is_connected()`, missing `WasEverUsed()` + `IsConnectedAndIdle()` pattern.

**Implementation**:
```rust
// src/socket/client.rs
impl SocketType {
    pub fn was_ever_used(&self) -> bool { self.was_used }
    
    pub fn is_usable(&mut self) -> Result<(), NetError> {
        if self.was_ever_used() {
            if !self.is_connected_and_idle()? {
                if !self.is_connected()? {
                    return Err(NetError::SocketRemoteClosed);
                }
                return Err(NetError::DataReceivedUnexpectedly);
            }
        } else if !self.is_connected()? {
            return Err(NetError::SocketRemoteClosed);
        }
        Ok(())
    }
}
```

**Files to modify**: `socket/pool.rs`, `socket/client.rs`

---

### 1.2 Content-Length Mismatch

**Chromium Pattern** (`http_stream_parser.cc:769`):
```cpp
result = ERR_CONTENT_LENGTH_MISMATCH;
```

**Current chromenet**: No body reading implemented yet — must add this when body support is added.

**Implementation**:
```rust
// src/http/body.rs
pub struct ResponseBody {
    remaining: Option<u64>,  // From Content-Length
    
    pub async fn read_chunk(&mut self) -> Result<Option<Bytes>, NetError> {
        let chunk = self.inner.read().await?;
        if chunk.is_empty() {
            if let Some(remaining) = self.remaining {
                if remaining > 0 {
                    return Err(NetError::ContentLengthMismatch);
                }
            }
        }
        if let Some(ref mut remaining) = self.remaining {
            *remaining = remaining.saturating_sub(chunk.len() as u64);
        }
        Ok(Some(chunk))
    }
}
```

**Files to add**: `http/body.rs`

---

### 1.3 Handle 1xx Informational Responses

**Chromium Pattern** (`http_network_transaction.cc:1619-1625`):
```cpp
if (response_.headers->response_code() / 100 == 1 && !ForWebSocketHandshake()) {
    response_.headers = base::MakeRefCounted<HttpResponseHeaders>(std::string());
    next_state_ = STATE_READ_HEADERS;
    return OK;
}
```

**Current chromenet**: No handling of 1xx responses.

**Implementation**:
```rust
// src/http/transaction.rs
async fn do_loop(&mut self) -> Result<(), NetError> {
    // In ReadHeaders state:
    loop {
        let response = self.read_headers().await?;
        match response.status().as_u16() {
            100..=199 => continue,  // Skip 1xx, keep reading
            _ => {
                self.response = Some(response);
                break;
            }
        }
    }
}
```

**Files to modify**: `http/transaction.rs`

---

## Priority 2: High (Security/Correctness)

### 2.1 Cookie Prefix Validation (__Secure-, __Host-)

**Chromium Pattern** (`canonical_cookie.cc:1222`):
```cpp
const std::string_view secure_prefix = "__Secure-";
// A __Secure- cookie must be Secure
// A __Host- cookie must be Secure, Path=/, no Domain
```

**Implementation**:
```rust
// src/cookies/canonical_cookie.rs
impl CanonicalCookie {
    pub fn validate_prefix(&self) -> Result<(), CookieError> {
        if self.name.starts_with("__Secure-") {
            if !self.secure {
                return Err(CookieError::InvalidSecurePrefix);
            }
        }
        if self.name.starts_with("__Host-") {
            if !self.secure || self.path != "/" || !self.host_only {
                return Err(CookieError::InvalidHostPrefix);
            }
        }
        Ok(())
    }
}
```

**Files to modify**: `cookies/canonical_cookie.rs`, `cookies/monster.rs`

---

### 2.2 Public Suffix List (PSL) Validation

**Current chromenet**: Uses simple domain suffix matching — allows `domain=.com`!

**Implementation**:
```rust
// Add dependency: psl = "2.0"
// src/cookies/monster.rs
use psl::List;

fn is_valid_cookie_domain(cookie_domain: &str, url_domain: &str) -> bool {
    let list = List::default();
    
    // Reject if cookie tries to set on public suffix
    if list.suffix(&cookie_domain.trim_start_matches('.'))
           .map(|s| s.typ().is_some())
           .unwrap_or(false) {
        return false;  // domain=.com, domain=.co.uk rejected
    }
    
    // Normal domain matching...
}
```

**Files to modify**: `cookies/monster.rs`, `Cargo.toml`

---

### 2.3 Redirect Limit and Loop Detection

**Chromium Pattern** (`url_request.cc:644`):
```cpp
redirect_limit_(kMaxRedirects),  // kMaxRedirects = 20
```

**Implementation**:
```rust
// src/urlrequest/job.rs
const MAX_REDIRECTS: u32 = 20;

struct RedirectChain {
    visited: HashSet<Url>,
    count: u32,
}

impl RedirectChain {
    fn check(&mut self, url: &Url) -> Result<(), NetError> {
        if self.count >= MAX_REDIRECTS {
            return Err(NetError::TooManyRedirects);
        }
        if !self.visited.insert(url.clone()) {
            return Err(NetError::RedirectCycleDetected);
        }
        self.count += 1;
        Ok(())
    }
}
```

**Files to modify**: `urlrequest/job.rs`

---

### 2.4 Strip Credentials on Cross-Origin Redirect

**Security**: CVE-2014-1829 (credentials leaked on redirect)

**Implementation**:
```rust
// src/urlrequest/job.rs
fn sanitize_redirect_url(original: &Url, redirect: &mut Url) {
    if redirect.origin() != original.origin() {
        // Cross-origin: strip credentials
        let _ = redirect.set_username("");
        let _ = redirect.set_password(None);
    }
}
```

---

## Priority 3: Medium (Robustness)

### 3.1 Transfer-Encoding vs Content-Length Precedence

**RFC 7230**: If both present, Transfer-Encoding takes precedence.

```rust
// src/http/transaction.rs
fn determine_body_length(headers: &HeaderMap) -> BodyLength {
    if headers.contains_key("transfer-encoding") {
        return BodyLength::Chunked;  // Ignore Content-Length
    }
    if let Some(cl) = headers.get("content-length") {
        if let Ok(len) = cl.to_str().and_then(|s| s.parse()) {
            return BodyLength::Known(len);
        }
    }
    BodyLength::Unknown
}
```

---

### 3.2 Redirect Method Change (POST → GET)

**RFC 7231**: 301/302 may, 303 must, 307/308 must not change method.

```rust
fn adjust_method_for_redirect(original: Method, status: u16) -> Method {
    match (status, original) {
        (301 | 302, Method::POST) => Method::GET,  // Historical behavior
        (303, _) => Method::GET,
        (307 | 308, method) => method,  // Preserve
        _ => original,
    }
}
```

---

### 3.3 SNI for IP Addresses

**RFC 6066**: Must NOT send SNI for raw IP addresses.

```rust
// src/socket/tls.rs
fn configure_sni(ssl_config: &mut SslConnectorBuilder, host: &str) {
    // Only set SNI for hostnames, not IPs
    if host.parse::<std::net::IpAddr>().is_err() {
        ssl_config.set_hostname(host).ok();
    }
}
```

---

## Priority 4: Low (Polish)

### 4.1 Timeout Granularity

```rust
struct Timeouts {
    connect: Duration,      // TCP + TLS
    read_idle: Duration,    // Between chunks
    total: Duration,        // End-to-end
}
```

### 4.2 Proxy 407 Authentication

Handle `407 Proxy Authentication Required` differently from `401`.

---

## Current Gap Analysis

| Edge Case | Status | Priority |
|-----------|--------|----------|
| Dead socket detection | ⚠️ Partial | P0 |
| Content-Length mismatch | ❌ Missing | P0 |
| 1xx response handling | ❌ Missing | P0 |
| __Secure-/__Host- prefix | ❌ Missing | P1 |
| Public suffix validation | ❌ Missing | P1 |
| Redirect limits | ❌ Missing | P1 |
| Credential stripping | ❌ Missing | P1 |
| Transfer-Encoding precedence | ❌ Missing | P2 |
| Redirect method change | ❌ Missing | P2 |
| SNI for IPs | ⚠️ Unknown | P2 |
| Timeout granularity | ❌ Missing | P3 |
| Proxy 407 auth | ❌ Missing | P3 |

---

## Recommended Order

1. **Add body support** (required for Content-Length checks)
2. **Dead socket detection** (pool.rs changes)
3. **1xx response handling** (transaction.rs)
4. **Cookie prefix validation** (canonical_cookie.rs)
5. **Redirect handling** (job.rs)
6. **Public suffix list** (add dependency + monster.rs)
