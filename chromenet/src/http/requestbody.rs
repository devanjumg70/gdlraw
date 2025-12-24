//! Request body for POST/PUT operations.

use bytes::Bytes;

/// Request body for HTTP methods that send data.
#[derive(Debug, Clone, Default)]
pub enum RequestBody {
    /// No body (GET, HEAD, DELETE).
    #[default]
    Empty,
    /// Body with raw bytes.
    Bytes(Bytes),
}

impl From<String> for RequestBody {
    fn from(s: String) -> Self {
        RequestBody::Bytes(Bytes::from(s))
    }
}

impl From<Vec<u8>> for RequestBody {
    fn from(v: Vec<u8>) -> Self {
        RequestBody::Bytes(Bytes::from(v))
    }
}

impl From<&str> for RequestBody {
    fn from(s: &str) -> Self {
        RequestBody::Bytes(Bytes::from(s.to_owned()))
    }
}

impl From<Bytes> for RequestBody {
    fn from(b: Bytes) -> Self {
        RequestBody::Bytes(b)
    }
}

impl RequestBody {
    /// Check if the body is empty.
    pub fn is_empty(&self) -> bool {
        matches!(self, RequestBody::Empty)
    }

    /// Get the length of the body in bytes.
    pub fn len(&self) -> usize {
        match self {
            RequestBody::Empty => 0,
            RequestBody::Bytes(b) => b.len(),
        }
    }
}
