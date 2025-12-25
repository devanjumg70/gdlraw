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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_body() {
        let body = RequestBody::Empty;
        assert!(body.is_empty());
        assert_eq!(body.len(), 0);
    }

    #[test]
    fn test_bytes_body() {
        let body = RequestBody::Bytes(Bytes::from("hello"));
        assert!(!body.is_empty());
        assert_eq!(body.len(), 5);
    }

    #[test]
    fn test_from_string() {
        let body: RequestBody = "hello world".to_string().into();
        assert_eq!(body.len(), 11);
    }

    #[test]
    fn test_from_str() {
        let body: RequestBody = "test".into();
        assert_eq!(body.len(), 4);
    }

    #[test]
    fn test_from_vec() {
        let body: RequestBody = vec![1u8, 2, 3, 4].into();
        assert_eq!(body.len(), 4);
    }

    #[test]
    fn test_from_bytes() {
        let body: RequestBody = Bytes::from_static(b"raw").into();
        assert_eq!(body.len(), 3);
    }

    #[test]
    fn test_default_is_empty() {
        let body = RequestBody::default();
        assert!(body.is_empty());
    }

    #[test]
    fn test_clone() {
        let body1: RequestBody = "data".into();
        let body2 = body1.clone();
        assert_eq!(body1.len(), body2.len());
    }
}
