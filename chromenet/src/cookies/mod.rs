//! Cookie management and browser cookie extraction.
//!
//! This module provides a complete cookie management system including:
//!
//! - **Storage**: In-memory cookie jar (`CookieMonster`)
//! - **Browser Extraction**: Read cookies from Chrome, Firefox, Safari, Edge, Brave, Opera
//! - **Decryption**: Platform-specific decryption (v10/v11 on Linux, Keychain on macOS, DPAPI on Windows)
//! - **Persistence**: Save/load cookies to disk
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
//! # Supported Browsers
//!
//! | Browser | Linux | macOS | Windows |
//! |---------|-------|-------|---------|
//! | Chrome/Chromium | v10, v11 | Keychain | DPAPI |
//! | Firefox | ✓ (plaintext) | ✓ | ✓ |
//! | Edge/Brave/Opera | v10, v11 | Keychain | DPAPI |
//! | Safari | N/A | ✓ (binary) | N/A |

pub mod browser;
pub mod canonical_cookie;
pub mod decrypt;
pub mod error;
pub mod monster;
pub mod oscrypt;
pub mod persistence;
pub mod psl;
pub mod safari;
