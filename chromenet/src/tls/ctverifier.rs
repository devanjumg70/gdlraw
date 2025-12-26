//! Multi-Log Certificate Transparency Verifier.
//!
//! Verifies Signed Certificate Timestamps (SCTs) against known CT logs.
//! Mirrors Chromium's `net/cert/multi_log_ct_verifier.cc`.
//!
//! ## CT Log Requirements
//! To verify SCTs, you need to configure known CT logs with their public keys.
//! Google publishes the list of known logs at:
//! https://www.gstatic.com/ct/log_list/v3/all_logs_list.json

use crate::base::neterror::NetError;
use crate::tls::ct::{CtRequirement, Sct, SctStatus};
use dashmap::DashMap;
use std::sync::Arc;
use time::OffsetDateTime;

/// Information about a known CT log.
#[derive(Debug, Clone)]
pub struct CtLog {
    /// Log ID (SHA-256 hash of the log's public key, 32 bytes)
    pub id: [u8; 32],
    /// DER-encoded public key (ECDSA P-256)
    pub public_key: Vec<u8>,
    /// Human-readable description
    pub description: String,
    /// Log operator
    pub operator: String,
}

impl CtLog {
    /// Create a new CT log entry.
    pub fn new(id: [u8; 32], public_key: Vec<u8>, description: impl Into<String>) -> Self {
        Self {
            id,
            public_key,
            description: description.into(),
            operator: String::new(),
        }
    }

    /// Set the operator name.
    pub fn with_operator(mut self, operator: impl Into<String>) -> Self {
        self.operator = operator.into();
        self
    }
}

/// Multi-log CT verifier.
///
/// Maintains a registry of known CT logs and verifies SCTs against them.
/// Mirrors Chromium's `MultiLogCTVerifier`.
///
/// ## Usage
/// ```ignore
/// let verifier = MultiLogCtVerifier::new();
/// verifier.add_log(log);
///
/// let results = verifier.verify(&scts, cert_der, OffsetDateTime::now_utc());
/// ```
pub struct MultiLogCtVerifier {
    /// Map of Log ID -> Log info
    logs: Arc<DashMap<[u8; 32], CtLog>>,
    /// CT requirement level
    requirement: CtRequirement,
}

impl Default for MultiLogCtVerifier {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiLogCtVerifier {
    /// Create a new empty CT verifier.
    pub fn new() -> Self {
        Self {
            logs: Arc::new(DashMap::new()),
            requirement: CtRequirement::SoftFail,
        }
    }

    /// Set the CT requirement level.
    pub fn with_requirement(mut self, requirement: CtRequirement) -> Self {
        self.requirement = requirement;
        self
    }

    /// Add a known CT log.
    pub fn add_log(&self, log: CtLog) {
        self.logs.insert(log.id, log);
    }

    /// Get the number of known logs.
    pub fn log_count(&self) -> usize {
        self.logs.len()
    }

    /// Check if a log ID is known.
    pub fn has_log(&self, log_id: &[u8; 32]) -> bool {
        self.logs.contains_key(log_id)
    }

    /// Verify SCTs against known logs.
    ///
    /// # Arguments
    /// * `scts` - List of SCTs to verify
    /// * `cert_der` - The DER-encoded certificate (used for signature verification)
    /// * `current_time` - Current time for timestamp validation
    ///
    /// # Returns
    /// List of verification results, one per SCT.
    pub fn verify(
        &self,
        scts: &[Sct],
        _cert_der: &[u8],
        current_time: OffsetDateTime,
    ) -> Vec<(Sct, SctStatus)> {
        let mut results = Vec::with_capacity(scts.len());

        for sct in scts {
            let status = self.verify_single_sct(sct, current_time);
            results.push((sct.clone(), status));
        }

        results
    }

    /// Verify a single SCT.
    fn verify_single_sct(&self, sct: &Sct, current_time: OffsetDateTime) -> SctStatus {
        // Look up the log by ID
        let Some(log) = self.logs.get(&sct.log_id) else {
            return SctStatus::UnknownLog;
        };

        // Check timestamp isn't in the future
        if sct.timestamp > current_time {
            return SctStatus::FutureTimestamp;
        }

        // Verify the signature
        if !self.verify_signature(sct, &log.public_key) {
            return SctStatus::InvalidSignature;
        }

        SctStatus::Valid
    }

    /// Verify the SCT signature using the log's public key.
    ///
    /// NOTE: This is a simplified implementation. Full verification requires:
    /// 1. Reconstructing the signed data (TBSCertificate or Precert)
    /// 2. Verifying the ECDSA signature over SHA-256 hash
    fn verify_signature(&self, sct: &Sct, public_key: &[u8]) -> bool {
        // Basic validation: signature and key must be non-empty
        if sct.signature.is_empty() || public_key.is_empty() {
            return false;
        }

        // TODO: Implement full ECDSA signature verification using boring crate
        // For now, we accept any non-empty signature from a known log
        // This matches the stub behavior but only for known logs

        true
    }

    /// Check if CT requirements are met.
    ///
    /// # Arguments
    /// * `results` - Verification results from `verify()`
    ///
    /// # Returns
    /// Ok if requirements are met, Err otherwise.
    pub fn check_requirements(&self, results: &[(Sct, SctStatus)]) -> Result<(), NetError> {
        match self.requirement {
            CtRequirement::NotRequired => Ok(()),
            CtRequirement::SoftFail => {
                // Log warning if no valid SCTs, but allow connection
                Ok(())
            }
            CtRequirement::Required => {
                // Need at least one valid SCT
                let has_valid = results
                    .iter()
                    .any(|(_, status)| *status == SctStatus::Valid);
                if has_valid {
                    Ok(())
                } else {
                    Err(NetError::CertificateTransparencyRequired)
                }
            }
        }
    }
}

/// Decode an SCT list from TLS extension bytes.
///
/// The SCT list format is:
/// - 2 bytes: total length of all SCTs
/// - For each SCT:
///   - 2 bytes: SCT length
///   - SCT data
pub fn decode_sct_list(data: &[u8]) -> Result<Vec<Sct>, NetError> {
    if data.len() < 2 {
        return Ok(Vec::new());
    }

    let total_len = u16::from_be_bytes([data[0], data[1]]) as usize;
    if data.len() < 2 + total_len {
        return Err(NetError::InvalidResponse);
    }

    let mut scts = Vec::new();
    let mut offset = 2;

    while offset + 2 <= 2 + total_len {
        let sct_len = u16::from_be_bytes([data[offset], data[offset + 1]]) as usize;
        offset += 2;

        if offset + sct_len > 2 + total_len {
            return Err(NetError::InvalidResponse);
        }

        if let Some(sct) = decode_single_sct(&data[offset..offset + sct_len]) {
            scts.push(sct);
        }

        offset += sct_len;
    }

    Ok(scts)
}

/// Decode a single SCT from bytes.
///
/// SCT format (RFC 6962):
/// - 1 byte: version (0 for v1)
/// - 32 bytes: log ID
/// - 8 bytes: timestamp (ms since epoch)
/// - 2 bytes: extensions length
/// - N bytes: extensions
/// - 2 bytes: signature length
/// - signature data
fn decode_single_sct(data: &[u8]) -> Option<Sct> {
    // Minimum size: 1 + 32 + 8 + 2 + 2 = 45 bytes
    if data.len() < 45 {
        return None;
    }

    // Version must be 0 (v1)
    if data[0] != 0 {
        return None;
    }

    // Log ID (32 bytes)
    let mut log_id = [0u8; 32];
    log_id.copy_from_slice(&data[1..33]);

    // Timestamp (8 bytes, ms since epoch)
    let timestamp_ms = u64::from_be_bytes([
        data[33], data[34], data[35], data[36], data[37], data[38], data[39], data[40],
    ]);
    let timestamp = OffsetDateTime::from_unix_timestamp((timestamp_ms / 1000) as i64).ok()?;

    // Extensions length (2 bytes)
    let ext_len = u16::from_be_bytes([data[41], data[42]]) as usize;

    // Verify we have enough data for extensions + signature
    let sig_offset = 43 + ext_len;
    if data.len() < sig_offset + 2 {
        return None;
    }

    // Signature (rest of data)
    let signature = data[sig_offset..].to_vec();

    Some(Sct {
        log_id,
        timestamp,
        signature,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_log() -> CtLog {
        let mut id = [0u8; 32];
        id[0] = 0x01;
        id[1] = 0x02;

        CtLog::new(id, vec![0x04, 0x00], "Test Log").with_operator("Test Operator")
    }

    #[test]
    fn test_add_log() {
        let verifier = MultiLogCtVerifier::new();
        let log = create_test_log();
        let id = log.id;

        verifier.add_log(log);

        assert_eq!(verifier.log_count(), 1);
        assert!(verifier.has_log(&id));
    }

    #[test]
    fn test_unknown_log() {
        let verifier = MultiLogCtVerifier::new();
        let sct = Sct {
            log_id: [0x99; 32],
            timestamp: OffsetDateTime::now_utc(),
            signature: vec![0x01, 0x02],
        };

        let results = verifier.verify(&[sct], &[], OffsetDateTime::now_utc());

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1, SctStatus::UnknownLog);
    }

    #[test]
    fn test_known_log_valid() {
        let verifier = MultiLogCtVerifier::new();
        let log = create_test_log();
        let log_id = log.id;
        verifier.add_log(log);

        let sct = Sct {
            log_id,
            timestamp: OffsetDateTime::now_utc() - time::Duration::hours(1),
            signature: vec![0x01, 0x02, 0x03],
        };

        let results = verifier.verify(&[sct], &[], OffsetDateTime::now_utc());

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1, SctStatus::Valid);
    }

    #[test]
    fn test_future_timestamp() {
        let verifier = MultiLogCtVerifier::new();
        let log = create_test_log();
        let log_id = log.id;
        verifier.add_log(log);

        let sct = Sct {
            log_id,
            timestamp: OffsetDateTime::now_utc() + time::Duration::hours(1),
            signature: vec![0x01, 0x02],
        };

        let results = verifier.verify(&[sct], &[], OffsetDateTime::now_utc());

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1, SctStatus::FutureTimestamp);
    }

    #[test]
    fn test_check_requirements_not_required() {
        let verifier = MultiLogCtVerifier::new().with_requirement(CtRequirement::NotRequired);
        let results = vec![];

        assert!(verifier.check_requirements(&results).is_ok());
    }

    #[test]
    fn test_check_requirements_required_with_valid() {
        let verifier = MultiLogCtVerifier::new().with_requirement(CtRequirement::Required);

        let sct = Sct {
            log_id: [0; 32],
            timestamp: OffsetDateTime::now_utc(),
            signature: vec![],
        };
        let results = vec![(sct, SctStatus::Valid)];

        assert!(verifier.check_requirements(&results).is_ok());
    }

    #[test]
    fn test_check_requirements_required_without_valid() {
        let verifier = MultiLogCtVerifier::new().with_requirement(CtRequirement::Required);

        let sct = Sct {
            log_id: [0; 32],
            timestamp: OffsetDateTime::now_utc(),
            signature: vec![],
        };
        let results = vec![(sct, SctStatus::UnknownLog)];

        assert!(verifier.check_requirements(&results).is_err());
    }

    #[test]
    fn test_decode_empty_sct_list() {
        let result = decode_sct_list(&[0, 0]);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
