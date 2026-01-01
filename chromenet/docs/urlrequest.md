# URLRequest Module

## Files
| File | Lines | Purpose |
|------|-------|---------|
| [context.rs](../src/urlrequest/context.rs) | ~150 | URLRequestContext configuration |
| [request.rs](../src/urlrequest/request.rs) | ~100 | URLRequest public API |
| [job.rs](../src/urlrequest/job.rs) | ~240 | HTTP job and redirect handling |
| [device.rs](../src/urlrequest/device.rs) | ~160 | Device emulation registry |
| [profile.rs](../src/urlrequest/profile.rs) | ~330 | Connection profile management |

---

## URLRequestContext (NEW)

Central configuration for all network operations, mirroring Chromium's `net::URLRequestContext`.

```rust
pub struct URLRequestContext {
    stream_factory: Arc<HttpStreamFactory>,
    socket_pool: Arc<ClientSocketPool>,
    cookie_store: Arc<CookieMonster>,
    config: URLRequestContextConfig,
}
```

### Configuration
```rust
pub struct URLRequestContextConfig {
    pub user_agent: String,
    pub accept_language: Option<String>,
    pub proxy: Option<ProxySettings>,
    pub max_sockets_per_group: usize,  // Default: 6
    pub max_sockets_total: usize,      // Default: 256
}
```

### Usage
```rust
let config = URLRequestContextConfig {
    user_agent: "MyApp/1.0".to_string(),
    proxy: Some(ProxySettings::new("socks5://localhost:1080").unwrap()),
    ..Default::default()
};
let context = URLRequestContext::with_config(config);
```

---

## URLRequest

Public-facing API for making HTTP requests.

```rust
let mut request = URLRequest::new("https://example.com")?;
request.set_device(DeviceRegistry::get_by_title("iPhone 12 Pro").unwrap());
request.set_proxy(ProxySettings::new("http://proxy:8080").unwrap());
request.add_header("Accept-Language", "en-US");
request.start().await?;
let response = request.get_response();
```

---

## URLRequestHttpJob

Handles redirect following and authentication.

### Redirect Logic
1. Check for 3xx status codes
2. Parse `Location` header (supports relative URLs)
3. Decrement `redirect_limit` (default: 20)
4. Strip `Authorization` on cross-origin redirect
5. Persist proxy settings and custom headers across redirects

---

## Device & DeviceRegistry

Emulated device definitions from Chromium's DevTools.

### Available Devices
- iPhone 12 Pro, iPhone 14 Pro Max
- Pixel 7, Samsung Galaxy S8+
- Desktop Chrome

> [!TIP]
> `DeviceRegistry::all()` calls are **cached** using `OnceLock`, making repeated lookups extremely fast (zero-allocation).

> [!TIP]
> Use `DeviceRegistry::get_by_title("Device Name")` to get a device.
