//! HTTP Digest Authentication (RFC 7616).
//!
//! Implements the Digest access authentication scheme as defined in RFC 7616.
//! Mirrors Chromium's `net/http/http_auth_handler_digest.cc`.
//!
//! ## Supported Features
//! - MD5 and SHA-256 algorithms
//! - qop=auth (quality of protection)
//! - Nonce count tracking for replay protection
//! - Session-based algorithms (MD5-sess, SHA-256-sess)

use crate::base::neterror::NetError;
use boring::hash::{hash, MessageDigest};
use std::fmt::Write;

/// Digest authentication algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DigestAlgorithm {
    /// Unspecified - defaults to MD5
    #[default]
    Unspecified,
    /// MD5
    Md5,
    /// MD5-sess (session-based)
    Md5Sess,
    /// SHA-256
    Sha256,
    /// SHA-256-sess (session-based)
    Sha256Sess,
}

impl DigestAlgorithm {
    /// Parse algorithm from header value.
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "md5" => Some(Self::Md5),
            "md5-sess" => Some(Self::Md5Sess),
            "sha-256" => Some(Self::Sha256),
            "sha-256-sess" => Some(Self::Sha256Sess),
            _ => None,
        }
    }

    /// Get the algorithm name for the Authorization header.
    fn as_str(&self) -> &'static str {
        match self {
            Self::Unspecified => "",
            Self::Md5 => "MD5",
            Self::Md5Sess => "MD5-sess",
            Self::Sha256 => "SHA-256",
            Self::Sha256Sess => "SHA-256-sess",
        }
    }

    /// Check if this is a session-based algorithm.
    fn is_session(&self) -> bool {
        matches!(self, Self::Md5Sess | Self::Sha256Sess)
    }
}

/// Quality of Protection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Qop {
    /// Unspecified
    #[default]
    Unspecified,
    /// Authentication only
    Auth,
    /// Authentication with integrity
    AuthInt,
}

impl Qop {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Unspecified => "",
            Self::Auth => "auth",
            Self::AuthInt => "auth-int",
        }
    }
}

/// HTTP Digest authentication handler.
///
/// Parses WWW-Authenticate challenges and generates Authorization headers.
/// Mirrors Chromium's `HttpAuthHandlerDigest`.
#[derive(Debug, Clone)]
pub struct DigestAuthHandler {
    /// The realm from the challenge
    pub realm: String,
    /// The original (unprocessed) realm for response generation
    original_realm: String,
    /// Server nonce
    nonce: String,
    /// Opaque value (passed back unchanged)
    opaque: Option<String>,
    /// Domain (unused but parsed)
    domain: Option<String>,
    /// Algorithm to use
    algorithm: DigestAlgorithm,
    /// Quality of protection
    qop: Qop,
    /// Stale flag (nonce expired but credentials still valid)
    stale: bool,
    /// Whether to use userhash
    userhash: bool,
    /// Current nonce count (incremented per request with same nonce)
    nonce_count: u32,
}

impl DigestAuthHandler {
    /// Parse a WWW-Authenticate: Digest challenge header.
    ///
    /// # Arguments
    /// * `header` - The header value after "Digest " prefix
    ///
    /// # Example
    /// ```ignore
    /// let handler = DigestAuthHandler::parse_challenge(
    ///     r#"realm="test", nonce="abc123", qop="auth", algorithm=MD5"#
    /// )?;
    /// ```
    pub fn parse_challenge(header: &str) -> Result<Self, NetError> {
        let mut handler = Self {
            realm: String::new(),
            original_realm: String::new(),
            nonce: String::new(),
            opaque: None,
            domain: None,
            algorithm: DigestAlgorithm::default(),
            qop: Qop::default(),
            stale: false,
            userhash: false,
            nonce_count: 0,
        };

        // Parse key=value pairs
        for part in Self::split_challenge(header) {
            let (key, value) = Self::parse_param(part)?;
            match key.to_lowercase().as_str() {
                "realm" => {
                    handler.original_realm = value.to_string();
                    handler.realm = value.to_string();
                }
                "nonce" => handler.nonce = value.to_string(),
                "opaque" => handler.opaque = Some(value.to_string()),
                "domain" => handler.domain = Some(value.to_string()),
                "algorithm" => {
                    handler.algorithm =
                        DigestAlgorithm::from_str(value).ok_or(NetError::InvalidResponse)?;
                }
                "qop" => {
                    // Parse comma-separated qop values, prefer "auth"
                    for qop_val in value.split(',') {
                        let qop_val = qop_val.trim();
                        if qop_val.eq_ignore_ascii_case("auth") {
                            handler.qop = Qop::Auth;
                            break;
                        }
                    }
                }
                "stale" => handler.stale = value.eq_ignore_ascii_case("true"),
                "userhash" => handler.userhash = value.eq_ignore_ascii_case("true"),
                _ => {} // Ignore unknown parameters
            }
        }

        // Nonce is required
        if handler.nonce.is_empty() {
            return Err(NetError::InvalidResponse);
        }

        Ok(handler)
    }

    /// Split challenge into individual parameters.
    fn split_challenge(header: &str) -> Vec<&str> {
        let mut parts = Vec::new();
        let mut start = 0;
        let mut in_quotes = false;

        for (i, c) in header.char_indices() {
            match c {
                '"' => in_quotes = !in_quotes,
                ',' if !in_quotes => {
                    let part = header[start..i].trim();
                    if !part.is_empty() {
                        parts.push(part);
                    }
                    start = i + 1;
                }
                _ => {}
            }
        }

        // Don't forget the last part
        let part = header[start..].trim();
        if !part.is_empty() {
            parts.push(part);
        }

        parts
    }

    /// Parse a single key=value or key="value" parameter.
    fn parse_param(param: &str) -> Result<(&str, &str), NetError> {
        let eq_pos = param.find('=').ok_or(NetError::InvalidResponse)?;
        let key = param[..eq_pos].trim();
        let mut value = param[eq_pos + 1..].trim();

        // Remove quotes if present
        if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
            value = &value[1..value.len() - 1];
        }

        Ok((key, value))
    }

    /// Generate the Authorization header value.
    ///
    /// # Arguments
    /// * `method` - HTTP method (GET, POST, etc.)
    /// * `uri` - Request URI path
    /// * `username` - Username for authentication
    /// * `password` - Password for authentication
    ///
    /// # Returns
    /// The complete Authorization header value (including "Digest " prefix).
    pub fn generate_auth_token(
        &mut self,
        method: &str,
        uri: &str,
        username: &str,
        password: &str,
    ) -> String {
        self.nonce_count += 1;
        let nc = format!("{:08x}", self.nonce_count);

        // Generate client nonce (16 hex chars like Chromium)
        let cnonce = self.generate_cnonce();

        // Compute response digest
        let response = self.compute_response(method, uri, username, password, &cnonce, &nc);

        // Build Authorization header
        self.assemble_credentials(username, uri, &response, &cnonce, &nc)
    }

    /// Generate a random client nonce.
    fn generate_cnonce(&self) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        format!("{:016x}", seed)
    }

    /// Compute the response digest.
    fn compute_response(
        &self,
        method: &str,
        uri: &str,
        username: &str,
        password: &str,
        cnonce: &str,
        nc: &str,
    ) -> String {
        // HA1 = H(user:realm:password)
        let ha1_input = format!("{}:{}:{}", username, self.original_realm, password);
        let mut ha1 = self.hex_hash(&ha1_input);

        // For session algorithms: HA1 = H(H(user:realm:pass):nonce:cnonce)
        if self.algorithm.is_session() {
            let sess_input = format!("{}:{}:{}", ha1, self.nonce, cnonce);
            ha1 = self.hex_hash(&sess_input);
        }

        // HA2 = H(method:uri)
        let ha2_input = format!("{}:{}", method, uri);
        let ha2 = self.hex_hash(&ha2_input);

        // Response calculation depends on qop
        let response_input = if self.qop != Qop::Unspecified {
            format!(
                "{}:{}:{}:{}:{}:{}",
                ha1,
                self.nonce,
                nc,
                cnonce,
                self.qop.as_str(),
                ha2
            )
        } else {
            format!("{}:{}:{}", ha1, self.nonce, ha2)
        };

        self.hex_hash(&response_input)
    }

    /// Compute hex-encoded hash using the configured algorithm.
    fn hex_hash(&self, input: &str) -> String {
        let md = match self.algorithm {
            DigestAlgorithm::Sha256 | DigestAlgorithm::Sha256Sess => MessageDigest::sha256(),
            _ => MessageDigest::md5(),
        };

        let digest = hash(md, input.as_bytes()).expect("hash should not fail");
        let mut hex = String::with_capacity(digest.len() * 2);
        for byte in digest.iter() {
            write!(hex, "{:02x}", byte).unwrap();
        }
        hex
    }

    /// Assemble the Authorization header value.
    fn assemble_credentials(
        &self,
        username: &str,
        uri: &str,
        response: &str,
        cnonce: &str,
        nc: &str,
    ) -> String {
        let mut auth = format!(
            "Digest username=\"{}\", realm=\"{}\", nonce=\"{}\", uri=\"{}\"",
            username, self.original_realm, self.nonce, uri
        );

        if self.algorithm != DigestAlgorithm::Unspecified {
            auth.push_str(&format!(", algorithm={}", self.algorithm.as_str()));
        }

        auth.push_str(&format!(", response=\"{}\"", response));

        if let Some(ref opaque) = self.opaque {
            auth.push_str(&format!(", opaque=\"{}\"", opaque));
        }

        if self.qop != Qop::Unspecified {
            auth.push_str(&format!(
                ", qop={}, nc={}, cnonce=\"{}\"",
                self.qop.as_str(),
                nc,
                cnonce
            ));
        }

        if self.userhash {
            auth.push_str(", userhash=true");
        }

        auth
    }

    /// Check if this challenge indicates stale credentials (nonce expired).
    pub fn is_stale(&self) -> bool {
        self.stale
    }

    /// Get the realm for this challenge.
    pub fn realm(&self) -> &str {
        &self.realm
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_challenge() {
        let challenge =
            r#"realm="testrealm@host.com", nonce="dcd98b7102dd2f0e8b11d0f600bfb0c093", qop="auth""#;
        let handler = DigestAuthHandler::parse_challenge(challenge).unwrap();

        assert_eq!(handler.realm, "testrealm@host.com");
        assert_eq!(handler.nonce, "dcd98b7102dd2f0e8b11d0f600bfb0c093");
        assert_eq!(handler.qop, Qop::Auth);
    }

    #[test]
    fn test_parse_with_algorithm() {
        let challenge = r#"realm="test", nonce="abc", algorithm=SHA-256"#;
        let handler = DigestAuthHandler::parse_challenge(challenge).unwrap();

        assert_eq!(handler.algorithm, DigestAlgorithm::Sha256);
    }

    #[test]
    fn test_parse_with_opaque() {
        let challenge = r#"realm="test", nonce="abc", opaque="xyz123""#;
        let handler = DigestAuthHandler::parse_challenge(challenge).unwrap();

        assert_eq!(handler.opaque, Some("xyz123".to_string()));
    }

    #[test]
    fn test_parse_stale() {
        let challenge = r#"realm="test", nonce="abc", stale=true"#;
        let handler = DigestAuthHandler::parse_challenge(challenge).unwrap();

        assert!(handler.is_stale());
    }

    #[test]
    fn test_missing_nonce_fails() {
        let challenge = r#"realm="test""#;
        assert!(DigestAuthHandler::parse_challenge(challenge).is_err());
    }

    #[test]
    fn test_generate_auth_token() {
        let challenge = r#"realm="test", nonce="abc123", qop="auth""#;
        let mut handler = DigestAuthHandler::parse_challenge(challenge).unwrap();

        let token = handler.generate_auth_token("GET", "/path", "user", "pass");

        // Verify the token contains expected parts
        assert!(token.starts_with("Digest username=\"user\""));
        assert!(token.contains("realm=\"test\""));
        assert!(token.contains("nonce=\"abc123\""));
        assert!(token.contains("uri=\"/path\""));
        assert!(token.contains("response=\""));
        assert!(token.contains("qop=auth"));
        assert!(token.contains("nc=00000001"));
    }

    #[test]
    fn test_nonce_count_increments() {
        let challenge = r#"realm="test", nonce="abc", qop="auth""#;
        let mut handler = DigestAuthHandler::parse_challenge(challenge).unwrap();

        let token1 = handler.generate_auth_token("GET", "/", "u", "p");
        let token2 = handler.generate_auth_token("GET", "/", "u", "p");

        assert!(token1.contains("nc=00000001"));
        assert!(token2.contains("nc=00000002"));
    }

    #[test]
    fn test_hex_hash_md5() {
        let handler = DigestAuthHandler {
            realm: String::new(),
            original_realm: String::new(),
            nonce: String::new(),
            opaque: None,
            domain: None,
            algorithm: DigestAlgorithm::Md5,
            qop: Qop::Unspecified,
            stale: false,
            userhash: false,
            nonce_count: 0,
        };

        // MD5("test") = 098f6bcd4621d373cade4e832627b4f6
        let result = handler.hex_hash("test");
        assert_eq!(result, "098f6bcd4621d373cade4e832627b4f6");
    }

    #[test]
    fn test_hex_hash_sha256() {
        let handler = DigestAuthHandler {
            realm: String::new(),
            original_realm: String::new(),
            nonce: String::new(),
            opaque: None,
            domain: None,
            algorithm: DigestAlgorithm::Sha256,
            qop: Qop::Unspecified,
            stale: false,
            userhash: false,
            nonce_count: 0,
        };

        // SHA-256("test") = 9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08
        let result = handler.hex_hash("test");
        assert_eq!(
            result,
            "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
        );
    }
}
