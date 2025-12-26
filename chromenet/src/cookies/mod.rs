//! Cookie management and browser cookie extraction.
//!
//! This module provides a complete cookie management system including:
//!
//! - **Storage**: In-memory cookie jar ([`CookieMonster`](monster::CookieMonster))
//! - **Browser Extraction**: Read cookies from Chrome, Firefox, Safari, Edge, Brave, Opera
//! - **Decryption**: Platform-specific decryption (v10/v11 on Linux, Keychain on macOS, DPAPI on Windows)
//! - **Persistence**: Save/load cookies to disk
//! - **Import/Export**: Netscape format and browser import
//!
//! # Architecture
//!
//! This implementation mirrors Chromium's cookie storage architecture:
//!
//! | Chromium (C++) | chromenet (Rust) | Responsibility |
//! |----------------|------------------|----------------|
//! | `net::CookieMonster` | [`CookieMonster`](monster::CookieMonster) | Cookie jar with LRU eviction |
//! | `net::CanonicalCookie` | [`CanonicalCookie`](canonical_cookie::CanonicalCookie) | Single cookie representation |
//! | `os_crypt::OSCrypt` | [`oscrypt`] | Cookie decryption |
//! | `SqlitePersistentCookieStore` | [`persistence`] | Disk persistence |
//!
//! # Browser Cookie Extraction
//!
//! ```rust,no_run
//! use chromenet::cookies::browser::{Browser, BrowserCookieReader};
//!
//! let reader = BrowserCookieReader::new(Browser::Chrome)
//!     .domain("example.com"); // Optional: filter by domain
//!
//! match reader.read_cookies_v2() {
//!     Ok(cookies) => println!("Found {} cookies", cookies.len()),
//!     Err(e) => eprintln!("Error: {:?}", e),
//! }
//! ```
//!
//! # Import from Browser into CookieMonster
//!
//! ```rust,no_run
//! use chromenet::cookies::monster::CookieMonster;
//! use chromenet::cookies::browser::Browser;
//!
//! let jar = CookieMonster::new();
//! jar.import_from_browser(Browser::Chrome, None)?;
//! println!("Imported {} cookies", jar.total_cookie_count());
//! # Ok::<(), chromenet::cookies::error::CookieExtractionError>(())
//! ```
//!
//! # Export to Netscape Format (curl/wget compatible)
//!
//! ```rust,no_run
//! use chromenet::cookies::monster::CookieMonster;
//!
//! let jar = CookieMonster::new();
//! // ... add cookies ...
//! let netscape = jar.export_netscape(None);
//! std::fs::write("cookies.txt", netscape)?;
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! # Supported Browsers
//!
//! | Browser | Linux | macOS | Windows |
//! |---------|-------|-------|---------|
//! | Chrome/Chromium | v10, v11 | Keychain | DPAPI |
//! | Firefox | ✓ (plaintext) | ✓ | ✓ |
//! | Edge/Brave/Opera | v10, v11 | Keychain | DPAPI |
//! | Safari | N/A | ✓ (binary) | N/A |
//!
//! # Chromium References
//!
//! - Database schema: `net/extras/sqlite/sqlite_persistent_cookie_store.cc`
//! - Encryption: `components/os_crypt/sync/os_crypt_linux.cc`
//! - Cookie monster: `net/cookies/cookie_monster.cc`

pub mod browser;
pub mod canonicalcookie;
pub mod chromedb;
pub mod decrypt;
pub mod error;
pub mod monster;
pub mod oscrypt;
pub mod persistence;
pub mod psl;
pub mod safari;
