//! Cookie management example.
//!
//! This example demonstrates using CookieMonster for session management.

use chromenet::cookies::monster::CookieMonster;
use url::Url;

fn main() {
    // Create a cookie jar
    let cookies = CookieMonster::new();

    // Simulate receiving a Set-Cookie header from a server
    let url = Url::parse("https://example.com/login").unwrap();
    cookies.parse_and_save_cookie(&url, "session_id=abc123; Path=/; Secure; HttpOnly");

    // Add another cookie
    cookies.parse_and_save_cookie(&url, "user_pref=dark_mode; Path=/; Max-Age=86400");

    // Get cookies for a subsequent request
    let cookies_for_request = cookies.get_cookies_for_url(&url);
    println!("Cookies for example.com:");
    for cookie in &cookies_for_request {
        println!("  {}={}", cookie.name, cookie.value);
    }

    // Demonstrate PSL validation (prevents supercookie attacks)
    // Trying to set a cookie on a public suffix should be rejected
    let psl_url = Url::parse("https://co.uk/").unwrap();
    cookies.parse_and_save_cookie(&psl_url, "evil=supercookie; Domain=.co.uk");

    // This cookie should be rejected because 'co.uk' is a public suffix
    let cookies_for_psl = cookies.get_cookies_for_url(&psl_url);
    println!(
        "\nCookies for public suffix co.uk (should be empty): {} cookies",
        cookies_for_psl.len()
    );
}
