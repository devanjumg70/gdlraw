//! Proxy configuration example.
//!
//! This example demonstrates how to configure different proxy types.

use chromenet::socket::proxy::ProxySettings;

fn main() {
    // HTTP Proxy
    let http_proxy =
        ProxySettings::new("http://proxy.example.com:8080").expect("Invalid proxy URL");
    println!("HTTP Proxy:");
    println!("  URL: {}", http_proxy.url);
    println!("  Type: {:?}", http_proxy.proxy_type());

    // HTTPS Proxy (TLS to proxy)
    let https_proxy =
        ProxySettings::new("https://secure-proxy.example.com:443").expect("Invalid proxy URL");
    println!("\nHTTPS Proxy:");
    println!("  URL: {}", https_proxy.url);
    println!("  Type: {:?}", https_proxy.proxy_type());

    // SOCKS5 Proxy
    let socks5_proxy = ProxySettings::new("socks5://localhost:1080").expect("Invalid proxy URL");
    println!("\nSOCKS5 Proxy:");
    println!("  URL: {}", socks5_proxy.url);
    println!("  Type: {:?}", socks5_proxy.proxy_type());

    // Proxy with authentication
    let auth_proxy = ProxySettings::new("http://proxy.example.com:8080")
        .expect("Invalid proxy URL")
        .with_auth("username", "password");

    if let Some(auth_header) = auth_proxy.get_auth_header() {
        println!("\nAuthenticated Proxy:");
        println!("  Proxy-Authorization: {}", auth_header);
    }
}
