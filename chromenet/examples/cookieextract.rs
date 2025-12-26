//! Example: Extract cookies from browser databases.
//!
//! This example demonstrates how to extract cookies from Chromium-based
//! browsers, Firefox, and Safari using the `chromenet` cookie extraction API.
//!
//! Usage: cargo run --example cookieextract
//!
//! Note: This requires access to browser cookie databases which may be locked
//! if the browser is running.

use chromenet::cookies::browser::{Browser, BrowserCookieReader};

fn main() {
    println!("=== Browser Cookie Extraction Example ===\n");

    // List of browsers to try
    let browsers = [
        Browser::Chrome,
        Browser::Chromium,
        Browser::Firefox,
        Browser::Brave,
        Browser::Edge,
        Browser::Opera,
    ];

    for browser in browsers {
        print_browser_cookies(browser, None);
    }
}

fn print_browser_cookies(browser: Browser, domain_filter: Option<&str>) {
    println!("--- {:?} ---", browser);

    // Create a reader with optional domain filter
    let mut reader = BrowserCookieReader::new(browser);
    if let Some(domain) = domain_filter {
        reader = reader.domain(domain);
    }

    // Check if cookie database exists
    match reader.get_db_path() {
        Some(path) => {
            if path.exists() {
                println!("  Database: {}", path.display());
            } else {
                println!("  Database not found: {}", path.display());
                println!();
                return;
            }
        }
        None => {
            println!("  Browser not installed or unsupported platform");
            println!();
            return;
        }
    }

    // Try to read cookies using the v2 API (better error handling)
    match reader.read_cookies_v2() {
        Ok(cookies) => {
            println!("  Found {} cookies", cookies.len());

            // Print first 5 cookies as sample
            for cookie in cookies.iter().take(5) {
                println!(
                    "    - {} = {}... (domain: {}, secure: {})",
                    cookie.name,
                    if cookie.value.len() > 20 {
                        format!("{}...", &cookie.value[..20])
                    } else {
                        cookie.value.clone()
                    },
                    cookie.domain,
                    cookie.secure
                );
            }

            if cookies.len() > 5 {
                println!("    ... and {} more", cookies.len() - 5);
            }
        }
        Err(e) => {
            println!("  Error: {:?}", e);
        }
    }

    println!();
}
