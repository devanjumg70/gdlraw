# Cookies Module

## Files
- [canonical_cookie.rs](file:///home/ubuntu/projects/chromium/dl/chromenet/src/cookies/canonical_cookie.rs) (70 lines)
- [monster.rs](file:///home/ubuntu/projects/chromium/dl/chromenet/src/cookies/monster.rs) (~230 lines)

---

## CanonicalCookie

Representation of a parsed cookie with all attributes.

```rust
pub struct CanonicalCookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub creation_time: OffsetDateTime,
    pub expiration_time: Option<OffsetDateTime>,
    pub last_access_time: OffsetDateTime,
    pub secure: bool,
    pub http_only: bool,
    pub host_only: bool,  // Exact match vs domain suffix
    pub same_site: SameSite,
    pub priority: CookiePriority,
}
```

### SameSite Values
- `Unspecified` - Browser default behavior
- `NoRestriction` - Cross-site allowed (requires `Secure`)
- `Lax` - Cross-site on safe requests
- `Strict` - Same-site only

---

## CookieMonster

Cookie store with **RFC 6265 domain matching** and **LRU eviction**.

### Features
| Feature | Status |
|---------|--------|
| Domain suffix matching | ✅ `.example.com` matches `sub.example.com` |
| Host-only cookies | ✅ Exact match only |
| Path matching | ✅ RFC 6265 prefix match |
| LRU eviction | ✅ 50 cookies per domain |
| Expiry checking | ✅ Expired cookies filtered |

### Storage Structure
```rust
store: Arc<DashMap<String, Vec<CanonicalCookie>>>  // Domain -> Cookies
```

### Key Methods
| Method | Description |
|--------|-------------|
| `set_canonical_cookie` | Store cookie with LRU eviction |
| `get_cookies_for_url` | Retrieve matching cookies (sorted by path) |
| `parse_and_save_cookie` | Parse `Set-Cookie` header and store |
| `total_cookie_count` | Get total cookies across all domains |
| `clear` | Remove all cookies |

### Limits
| Limit | Value | Status |
|-------|-------|--------|
| Per-domain | 50 cookies | ✅ Enforced |
| Total | 3000 cookies | ✅ Enforced |

---

## Persistence Module

Save and load cookies to/from JSON files.

```rust
// Save all cookies
persistence::save_cookies(&monster, Path::new("cookies.json"))?;

// Load cookies (filters expired)
let monster = persistence::load_cookies(Path::new("cookies.json"))?;
```
