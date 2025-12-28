//! HTTP protocol handling.
//!
//! Provides HTTP/1.1 and HTTP/2 support mirroring Chromium's `net/http/`:
//! - [`transaction`]: State machine for request/response lifecycle
//! - [`streamfactory`]: H1/H2 stream creation
//! - [`httpcache`]: In-memory HTTP cache with Cache-Control
//! - [`multipart`]: Multipart form data encoding
//! - [`responsebody`]: Body streaming with `futures::Stream`

pub mod digestauth;
pub mod h2fingerprint;
pub mod httpcache;
pub mod multipart;
pub mod orderedheaders;
pub mod requestbody;
pub mod response;
pub mod responsebody;
pub mod retry;
pub mod streamfactory;
pub mod transaction;

// Re-exports for convenience
pub use h2fingerprint::H2Fingerprint;
pub use httpcache::{CacheEntry, CacheMode, HttpCache};
pub use requestbody::RequestBody;
pub use response::HttpResponse;
pub use responsebody::ResponseBody;
