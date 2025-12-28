//! QUIC and HTTP/3 support.
//!
//! Provides QUIC transport and HTTP/3 protocol support using quinn.
//! Mirrors Chromium's net/quic/ implementation pattern.
//!
//! # Status
//! This module provides the types and API structure for HTTP/3 support.
//! Full implementation requires the `quinn` crate for QUIC transport.
//!
//! # Example
//! ```ignore
//! use chromenet::quic::{QuicConnection, H3Client};
//!
//! let conn = QuicConnection::connect("https://example.com").await?;
//! let client = H3Client::new(conn);
//! let response = client.get("https://example.com/api").await?;
//! ```

mod config;
mod connection;

pub use config::QuicConfig;
pub use connection::{QuicConnection, QuicConnectionBuilder};
