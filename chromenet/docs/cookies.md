# Cookies Module

## Files
| File | Lines | Purpose |
|------|-------|---------|
| [canonicalcookie.rs](../src/cookies/canonicalcookie.rs) | ~70 | Cookie data structure (Renamed from `canonical_cookie.rs`) |
| [monster.rs](../src/cookies/monster.rs) | ~270 | Cookie storage & matching |
| [persistence.rs](../src/cookies/persistence.rs) | ~50 | JSON save/load |
| [psl.rs](../src/cookies/psl.rs) | ~130 | Public Suffix List validation |
| [browser.rs](../src/cookies/browser.rs) | ~385 | Chrome/Firefox extraction |
| [oscrypt.rs](../src/cookies/oscrypt.rs) | ~145 | Chrome v10 decryption |

---

## Security

> [!IMPORTANT]
> **Zeroize Implementation**: Sensitive key material (decryption keys, passwords) is automatically zeroed out from memory when dropped using the `zeroize` crate. This applies to `AES` keys derived for cookie decryption.

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
| **PSL validation** | ✅ Rejects supercookie attacks |

### Limits
| Limit | Value | Status |
|-------|-------|--------|
| Per-domain | 50 cookies | ✅ Enforced |
| Total | 3000 cookies | ✅ Enforced |

---

## PSL Module (NEW)

Public Suffix List validation to prevent supercookie attacks.

```rust
use chromenet::cookies::psl::{is_public_suffix, is_valid_cookie_domain};

// Reject cookies on public suffixes
assert!(is_public_suffix("com"));
assert!(is_public_suffix("co.uk"));
assert!(is_public_suffix("github.io"));

// Validate cookie domain
assert!(is_valid_cookie_domain("example.com", "sub.example.com"));
assert!(!is_valid_cookie_domain(".com", "example.com")); // Rejected!
```

---

## Browser Cookie Extraction (NEW)

Read cookies from Chrome/Firefox SQLite databases.

```rust
use chromenet::cookies::browser::{Browser, BrowserCookieReader};

let reader = BrowserCookieReader::new(Browser::Chrome)
    .with_profile("Profile 1");

let cookies = reader.read_cookies()?;
for cookie in cookies {
    println!("{}: {}", cookie.name, cookie.value);
}
```

### Platform Paths
| Browser | OS | Path |
|---------|-----|------|
| Chrome | Linux | `~/.config/google-chrome/Default/Cookies` |
| Chrome | macOS | `~/Library/Application Support/Google/Chrome/Default/Cookies` |
| Chrome | Windows | `%LOCALAPPDATA%/Google/Chrome/User Data/Default/Network/Cookies` |
| Firefox | Linux | `~/.mozilla/firefox/*.default/cookies.sqlite` |

---

## oscrypt Module (NEW)

Decrypt Chrome's v10 encrypted cookies (Linux).

```rust
use chromenet::cookies::oscrypt::{decrypt_v10, is_encrypted};

if is_encrypted(encrypted_value) {
    if let Some(value) = decrypt_v10(encrypted_value) {
        println!("Decrypted: {}", value);
    }
}
```

### Encryption Versions
| Version | Key Source | Status |
|---------|------------|--------|
| v10 (Linux) | Hardcoded PBKDF2 | ✅ Implemented |
| v10 (Windows) | DPAPI | ⏳ Not implemented |
| v11 | Keyring/Keychain | ⏳ Not implemented |

---

## Persistence Module

Save and load cookies to/from JSON files.

```rust
// Save all cookies
persistence::save_cookies(&monster, Path::new("cookies.json"))?;

// Load cookies (filters expired)
let monster = persistence::load_cookies(Path::new("cookies.json"))?;
```
