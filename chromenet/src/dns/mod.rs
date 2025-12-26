//! DNS Resolution Module
//!
//! Provides pluggable DNS resolution with support for:
//! - System resolver (getaddrinfo via thread pool)
//! - Async hickory-dns resolver (DoH/DoT capable)
//! - Hostname-to-IP override mechanism
//!
//! # Architecture
//!
//! This module mirrors Chromium's `HostResolver` concept but with a cleaner
//! Rust-idiomatic design. The `Resolve` trait is the core abstraction that
//! allows different resolver implementations to be used interchangeably.
//!
//! # Example
//!
//! ```rust,ignore
//! use chromenet::dns::{Name, Resolve, HickoryResolver};
//!
//! let resolver = HickoryResolver::new();
//! let addrs = resolver.resolve(Name::new("example.com")).await?;
//! for addr in addrs {
//!     println!("Resolved: {}", addr);
//! }
//! ```

mod gai;
mod hickory;
mod resolve;

pub use gai::GaiResolver;
pub use hickory::HickoryResolver;
pub use resolve::{Addrs, DnsResolverWithOverrides, Name, Resolve, Resolving};
