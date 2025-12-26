//! Ergonomic error context helpers.
//!
//! Provides extension traits for adding context to `Result` types,
//! converting IO errors into context-rich `NetError` variants.

use crate::base::neterror::NetError;
use std::io;

/// Extension trait for adding context to IO Results.
pub trait IoResultExt<T> {
    /// Add connection context to an IO error.
    ///
    /// # Example
    /// ```ignore
    /// use chromenet::base::context::IoResultExt;
    ///
    /// let stream = TcpStream::connect(addr).await
    ///     .connection_context("example.com", 443)?;
    /// // Error: "Connection to example.com:443 failed: connection refused"
    /// ```
    fn connection_context(self, host: &str, port: u16) -> Result<T, NetError>;

    /// Add DNS resolution context to an IO error.
    fn dns_context(self, domain: &str) -> Result<T, NetError>;
}

impl<T> IoResultExt<T> for Result<T, io::Error> {
    fn connection_context(self, host: &str, port: u16) -> Result<T, NetError> {
        self.map_err(|e| NetError::connection_failed_to(host, port, e))
    }

    fn dns_context(self, domain: &str) -> Result<T, NetError> {
        self.map_err(|e| NetError::dns_failed(domain, e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Error, ErrorKind};

    #[test]
    fn test_connection_context() {
        let result: Result<(), io::Error> =
            Err(Error::new(ErrorKind::ConnectionRefused, "refused"));
        let err = result.connection_context("example.com", 443).unwrap_err();

        match err {
            NetError::ConnectionFailedTo { host, port, .. } => {
                assert_eq!(host, "example.com");
                assert_eq!(port, 443);
            }
            _ => panic!("Expected ConnectionFailedTo"),
        }
    }

    #[test]
    fn test_dns_context() {
        let result: Result<(), io::Error> = Err(Error::new(ErrorKind::NotFound, "no such host"));
        let err = result.dns_context("unknown.example.com").unwrap_err();

        match err {
            NetError::NameNotResolvedFor { domain, .. } => {
                assert_eq!(domain, "unknown.example.com");
            }
            _ => panic!("Expected NameNotResolvedFor"),
        }
    }
}
