//! # chromenet
//!
//! A Chromium-inspired HTTP networking library for Rust.
//!
//! `chromenet` provides a high-fidelity implementation of browser networking
//! behavior, including connection pooling, cookie management, TLS security,
//! and browser fingerprint emulation.
//!
//! ## Features
//!
//! - **Connection Pooling**: 6 connections per host limit (Chromium-compatible)
//! - **HTTP/1.1 & HTTP/2**: Full protocol support with multiplexing
//! - **Cookie Management**: RFC 6265 compliant with PSL validation
//! - **TLS Security**: BoringSSL, HSTS, certificate pinning
//! - **Browser Emulation**: Device profiles, ordered headers, H2 fingerprinting
//! - **Proxy Support**: HTTP, HTTPS, and SOCKS5 proxies
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use chromenet::urlrequest::URLRequest;
//! use chromenet::cookies::CookieMonster;
//!
//! #[tokio::main]
//! async fn main() {
//!     let cookies = CookieMonster::new();
//!     let response = URLRequest::new("https://example.com")
//!         .with_cookies(cookies)
//!         .send()
//!         .await
//!         .unwrap();
//!     println!("Status: {}", response.status());
//! }
//! ```
//!
//! ## Modules
//!
//! - [`base`] - Core types and error definitions
//! - [`cookies`] - Cookie storage, parsing, and browser extraction
//! - [`http`] - HTTP transactions, headers, and body handling
//! - [`socket`] - Connection pooling, proxy, and TLS sockets
//! - [`tls`] - HSTS, certificate pinning, and CT verification
//! - [`urlrequest`] - High-level request API and device emulation
//!
//! ## Security
//!
//! This library implements several security features from Chromium:
//! - Public Suffix List validation to prevent supercookie attacks
//! - HSTS enforcement with preloaded domains
//! - Certificate pinning with SPKI hash verification
//! - Redirect cycle detection and credential stripping

pub mod base;
pub mod cookies;
pub mod http;
pub mod socket;
pub mod tls;
pub mod urlrequest;
