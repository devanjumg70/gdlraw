//! Socket and connection management.
//!
//! Provides connection pooling and socket handling mirroring Chromium's `net/socket/`:
//! - [`pool`]: Connection pooling (6 per host, 256 total)
//! - [`connectjob`]: DNS → TCP → TLS connection flow
//! - [`proxy`]: HTTP/HTTPS/SOCKS5 proxy support
//! - [`tls`]: TLS configuration with BoringSSL

pub mod authcache;
pub mod client;
pub mod connectjob;
pub mod matcher;
pub mod pool;
pub mod proxy;
pub mod stream;
pub mod tls;
