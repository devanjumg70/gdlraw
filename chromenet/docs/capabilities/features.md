# Chromenet Features

Complete list of chromenet capabilities with API entry points and implementation details.

---

## Core Networking

### Connection Pooling
Chromium-style socket management with per-host and global limits.

| Config | Default | Description |
|--------|---------|-------------|
| Max per host | 6 | Matches Chromium's `kMaxSocketsPerGroup` |
| Max total | 256 | Matches Chromium's `kMaxSockets` |

**API**: `socket::pool::ClientSocketPool`
- Request queuing when limits reached
- Priority-based queue ordering
- Idle socket cleanup (5 min used, 10 sec unused)

### HTTP/1.1 & HTTP/2
Full protocol support with automatic negotiation.

**Features**:
- HTTP/2 multiplexing
- H2 session caching
- Automatic protocol upgrade via ALPN

**API**: `http::streamfactory::HttpStreamFactory`

### HTTP Caching
RFC 7234 compliant in-memory cache.

| Feature | Description |
|---------|-------------|
| Cache-Control | `max-age`, `no-store`, `no-cache`, `private` |
| Validation | ETag, Last-Modified, 304 handling |
| Eviction | LRU-style with configurable limits |

**API**: `http::httpcache::HttpCache`

---

## Browser Emulation

### TLS Fingerprinting
Fine-grained control over TLS ClientHello.

**30+ BoringSSL options including**:
- Cipher suite ordering
- Extension ordering
- ALPN protocols
- Signature algorithms
- Key exchange groups
- GREASE support

**API**: `socket::tls::TlsOptions`

### HTTP/2 Fingerprinting
Match browser-specific H2 behavior.

**Controllable parameters**:
- SETTINGS frame order
- Pseudo-header order (`:method`, `:path`, etc.)
- Priority frames
- Window size values

**API**: `http::h2fingerprint::H2Fingerprint`

### Browser Profiles
Pre-configured emulation settings for 67 browser variants.

| Browser | Versions | Count |
|---------|----------|-------|
| Chrome | V100-V143 | 21 |
| Firefox | V109-V145 | 13 |
| Safari | V15.0-V18.4 | 15 |
| Edge | V101-V142 | 10 |
| OkHttp | 3.x-5.x | 8 |

**API**: 
- `emulation::profiles::chrome::Chrome`
- `emulation::profiles::firefox::Firefox`
- `emulation::profiles::safari::Safari`
- `emulation::profiles::edge::Edge`

### Device Emulation
Mobile and desktop device simulation.

**Features**:
- User-Agent string
- Sec-CH-UA client hints
- Screen dimensions
- Device pixel ratio

**API**: `urlrequest::device::DeviceRegistry`

---

## Cookie Management

### CookieMonster
RFC 6265 compliant cookie storage.

**Features**:
- Domain/path matching
- Secure/HttpOnly flags
- SameSite enforcement
- Public Suffix List (PSL) validation
- LRU eviction

**API**: `cookies::monster::CookieMonster`

### Browser Cookie Import
Extract cookies from installed browsers.

| Browser | Linux | macOS | Windows |
|---------|-------|-------|---------|
| Chrome | ✅ | ✅ | ✅ |
| Firefox | ✅ | ✅ | ✅ |
| Safari | ❌ | ✅ | ❌ |
| Edge | ✅ | ✅ | ✅ |

**Decryption support**:
- Linux: GNOME Keyring (v11), hardcoded key (v10)
- macOS: Keychain + PBKDF2
- Windows: DPAPI + AES-256-GCM

**API**: `cookies::browser::import_from_browser()`

### Cookie Export
Export to Netscape format for curl/wget compatibility.

**API**: `cookies::monster::CookieMonster::export_netscape()`

---

## Security Features

### HSTS (HTTP Strict Transport Security)
Automatic HTTPS upgrade for known domains.

**Features**:
- Preload list support
- Dynamic HSTS from headers
- includeSubDomains
- JSON persistence

**API**: `tls::hsts::HstsStore`

### Certificate Pinning
SPKI hash verification for TLS certificates.

**Features**:
- Pin set per domain
- Subdomain inheritance
- Expiration handling
- Report-only mode

**API**: `tls::pinning::PinStore`

### Certificate Transparency
SCT (Signed Certificate Timestamp) validation with multi-log support.

**Features**:
- SCT list parsing (RFC 6962)
- Log registry with public key storage
- Timestamp validation (future-date rejection)
- Requirement levels (`NotRequired`, `SoftFail`, `Required`)

**API**: `tls::ctverifier::MultiLogCtVerifier`

> [!NOTE]
> ECDSA signature verification uses a placeholder; all non-empty signatures from known logs are accepted.

---

## Protocol Support

### WebSocket
Full WebSocket client via tokio-tungstenite.

**Features**:
- Text and binary messages
- Ping/pong handling
- Close handshake
- Custom headers
- Subprotocol negotiation

**API**: `ws::WebSocket`

### Multipart Uploads
Ergonomic multipart/form-data construction.

**API**: `http::multipart::Form`

```rust
let form = Form::new()
    .text("field", "value")
    .part("file", Part::bytes(data)
        .file_name("upload.bin")
        .content_type("application/octet-stream"));
```

### Streaming Bodies
Memory-efficient large response handling.

**API**: `http::responsebody::BodyStream`

---

## Proxy Support

### Supported Protocols
| Protocol | Authentication | TLS-in-TLS |
|----------|---------------|------------|
| HTTP | Basic | ✅ |
| HTTPS | Basic | ✅ |
| SOCKS5 | Username/Password | ✅ |

**Features**:
- Proxy rotation
- NO_PROXY environment variable
- Per-request proxy override

**API**: `socket::proxy::ProxySettings`

---

## DNS Resolution

### Async DNS
Hickory (formerly TrustDNS) based resolver.

**Features**:
- DNS caching
- IPv4/IPv6 support
- Custom nameservers
- Happy Eyeballs (RFC 8305) with 250ms fallback delay

**API**: `dns::resolver::HickoryResolver`
