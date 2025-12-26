//! Certificate Transparency (CT) verification.
//!
//! Validates Signed Certificate Timestamps (SCTs) to ensure certificates
//! were logged to public CT logs. Based on Chromium's net/cert/ct_verifier.h.

use crate::base::neterror::NetError;
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CtRequirement {
    /// CT not required (HTTP connections)
    NotRequired,
    /// CT required but may be missing (warning)
    SoftFail,
    /// CT required (connection fails without valid SCTs)
    Required,
}

/// Certificate Transparency verifier.
///
/// NOTE: This is a stub implementation. Full CT verification requires:
/// - Log public keys for signature verification
/// - Log inclusion proof verification
/// - SCT parsing from TLS extensions or OCSP responses
///
/// Chromium: net/cert/ct_verifier.h
pub struct CtVerifier {
    requirement: CtRequirement,
}

impl Default for CtVerifier {
    fn default() -> Self {
        Self::new()
    }
}

impl CtVerifier {
    /// Create a new CT verifier with soft-fail requirement.
    pub fn new() -> Self {
        Self {
            requirement: CtRequirement::SoftFail,
        }
    }

    /// Set the CT requirement level.
    pub fn with_requirement(mut self, requirement: CtRequirement) -> Self {
        self.requirement = requirement;
        self
    }

    /// Check if CT verification is required.
    pub fn is_required(&self) -> bool {
        self.requirement != CtRequirement::NotRequired
    }

    /// Verify SCTs for a certificate chain.
    ///
    /// Returns Ok if CT requirements are met or not required.
    /// Returns Err if required SCTs are missing/invalid.
    pub fn verify(&self, _cert_chain: &[&[u8]], scts: &[Sct]) -> Result<(), NetError> {
        match self.requirement {
            CtRequirement::NotRequired => Ok(()),
            CtRequirement::SoftFail => {
                // Log warning if no SCTs, but allow connection
                if scts.is_empty() {
                    // In real implementation: log warning
                }
                Ok(())
            }
            CtRequirement::Required => {
                if scts.is_empty() {
                    Err(NetError::NotImplemented)
                } else {
                    // Stub: accept any SCTs for now
                    Ok(())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ct_not_required() {
        let verifier = CtVerifier::new().with_requirement(CtRequirement::NotRequired);
        assert!(!verifier.is_required());
        assert!(verifier.verify(&[], &[]).is_ok());
    }

    #[test]
    fn test_ct_soft_fail() {
        let verifier = CtVerifier::new().with_requirement(CtRequirement::SoftFail);
        assert!(verifier.is_required());
        assert!(verifier.verify(&[], &[]).is_ok());
    }

    #[test]
    fn test_ct_required_no_scts() {
        let verifier = CtVerifier::new().with_requirement(CtRequirement::Required);
        assert!(verifier.verify(&[], &[]).is_err());
    }

    #[test]
    fn test_default_verifier() {
        let verifier = CtVerifier::default();
        assert!(verifier.is_required());
    }
}
