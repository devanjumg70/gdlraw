//! Certificate Transparency (CT) types and verification.
//!
//! Validates Signed Certificate Timestamps (SCTs) to ensure certificates
//! were logged to public CT logs. Based on Chromium's net/cert/ct_verifier.h.
//!
//! For the full multi-log verifier implementation, see [`ctverifier`](super::ctverifier).

use time::OffsetDateTime;

/// Signed Certificate Timestamp from a CT log.
#[derive(Debug, Clone)]
pub struct Sct {
    /// Log ID (32-byte SHA-256 hash of log's public key)
    pub log_id: [u8; 32],
    /// Timestamp when the SCT was issued
    pub timestamp: OffsetDateTime,
    /// SCT signature
    pub signature: Vec<u8>,
}

/// Result of SCT verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SctStatus {
    /// SCT is valid and from a known log
    Valid,
    /// SCT signature verification failed
    InvalidSignature,
    /// SCT is from an unknown log
    UnknownLog,
    /// SCT timestamp is in the future
    FutureTimestamp,
}

/// CT verification requirements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CtRequirement {
    /// CT not required (HTTP connections)
    NotRequired,
    /// CT required but may be missing (warning)
    #[default]
    SoftFail,
    /// CT required (connection fails without valid SCTs)
    Required,
}

/// Alias for backward compatibility.
///
/// Use [`MultiLogCtVerifier`](super::ctverifier::MultiLogCtVerifier) for full functionality.
#[deprecated(since = "0.2.0", note = "Use MultiLogCtVerifier instead")]
pub type CtVerifier = super::ctverifier::MultiLogCtVerifier;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sct_creation() {
        let sct = Sct {
            log_id: [0u8; 32],
            timestamp: OffsetDateTime::now_utc(),
            signature: vec![0x01, 0x02, 0x03],
        };
        assert_eq!(sct.log_id.len(), 32);
    }

    #[test]
    fn test_sct_status_eq() {
        assert_eq!(SctStatus::Valid, SctStatus::Valid);
        assert_ne!(SctStatus::Valid, SctStatus::UnknownLog);
    }

    #[test]
    fn test_ct_requirement_default() {
        assert_eq!(CtRequirement::default(), CtRequirement::SoftFail);
    }
}
