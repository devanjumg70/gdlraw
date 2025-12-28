//! WebSocket message types.

use bytes::Bytes;

/// WebSocket message type.
#[derive(Debug, Clone)]
pub enum Message {
    /// Text message (UTF-8)
    Text(String),
    /// Binary message
    Binary(Bytes),
    /// Ping frame
    Ping(Vec<u8>),
    /// Pong frame
    Pong(Vec<u8>),
    /// Close frame with optional code and reason
    Close(Option<CloseFrame>),
}

/// Close frame data.
#[derive(Debug, Clone)]
pub struct CloseFrame {
    /// Close code (RFC 6455)
    pub code: CloseCode,
    /// Close reason (optional UTF-8 string)
    pub reason: String,
}

impl CloseFrame {
    /// Create a new close frame.
    pub fn new(code: CloseCode, reason: impl Into<String>) -> Self {
        Self {
            code,
            reason: reason.into(),
        }
    }
}

/// WebSocket close codes (RFC 6455).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CloseCode(pub u16);

impl CloseCode {
    /// Normal closure
    pub const NORMAL: Self = Self(1000);
    /// Server going down
    pub const GOING_AWAY: Self = Self(1001);
    /// Protocol error
    pub const PROTOCOL_ERROR: Self = Self(1002);
    /// Unsupported data type
    pub const UNSUPPORTED: Self = Self(1003);
    /// No status received
    pub const NO_STATUS: Self = Self(1005);
    /// Abnormal closure
    pub const ABNORMAL: Self = Self(1006);
    /// Invalid payload data
    pub const INVALID_PAYLOAD: Self = Self(1007);
    /// Policy violation
    pub const POLICY_VIOLATION: Self = Self(1008);
    /// Message too big
    pub const MESSAGE_TOO_BIG: Self = Self(1009);
    /// Extension required
    pub const EXTENSION_REQUIRED: Self = Self(1010);
    /// Internal server error
    pub const INTERNAL_ERROR: Self = Self(1011);
    /// TLS handshake failure
    pub const TLS_HANDSHAKE: Self = Self(1015);
}

impl From<u16> for CloseCode {
    fn from(code: u16) -> Self {
        Self(code)
    }
}

impl From<CloseCode> for u16 {
    fn from(code: CloseCode) -> Self {
        code.0
    }
}

impl Message {
    /// Check if this is a text message.
    pub fn is_text(&self) -> bool {
        matches!(self, Message::Text(_))
    }

    /// Check if this is a binary message.
    pub fn is_binary(&self) -> bool {
        matches!(self, Message::Binary(_))
    }

    /// Check if this is a close message.
    pub fn is_close(&self) -> bool {
        matches!(self, Message::Close(_))
    }

    /// Try to get as text.
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Message::Text(s) => Some(s),
            _ => None,
        }
    }

    /// Try to get as binary data.
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Message::Binary(b) => Some(b),
            _ => None,
        }
    }

    /// Convert to bytes (text as UTF-8, binary as-is).
    pub fn into_data(self) -> Vec<u8> {
        match self {
            Message::Text(s) => s.into_bytes(),
            Message::Binary(b) => b.to_vec(),
            Message::Ping(d) | Message::Pong(d) => d,
            Message::Close(_) => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_types() {
        let text = Message::Text("hello".into());
        assert!(text.is_text());
        assert!(!text.is_binary());
        assert_eq!(text.as_text(), Some("hello"));

        let binary = Message::Binary(Bytes::from_static(b"data"));
        assert!(binary.is_binary());
        assert!(!binary.is_text());
    }

    #[test]
    fn test_close_codes() {
        assert_eq!(CloseCode::NORMAL.0, 1000);
        assert_eq!(CloseCode::GOING_AWAY.0, 1001);

        let code: u16 = CloseCode::NORMAL.into();
        assert_eq!(code, 1000);
    }

    #[test]
    fn test_close_frame() {
        let frame = CloseFrame::new(CloseCode::NORMAL, "bye");
        assert_eq!(frame.code, CloseCode::NORMAL);
        assert_eq!(frame.reason, "bye");
    }

    #[test]
    fn test_into_data() {
        let text = Message::Text("test".into());
        assert_eq!(text.into_data(), b"test");

        let binary = Message::Binary(Bytes::from_static(b"bin"));
        assert_eq!(binary.into_data(), b"bin");
    }
}
