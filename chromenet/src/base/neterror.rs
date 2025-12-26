use std::io;
use std::sync::Arc;
use thiserror::Error;

/// Network error type mirroring Chromium's net/base/net_error_list.h.
///
/// This enum covers all network-level errors including connection, SSL/TLS,
/// HTTP, and cookie extraction errors.
#[derive(Debug, Error, Clone)]
pub enum NetError {
    // Connection Errors
    #[error("Connection closed (TCP FIN)")]
    ConnectionClosed,
    #[error("Connection reset (TCP RST)")]
    ConnectionReset,
    #[error("Connection refused")]
    ConnectionRefused,
    #[error("Connection aborted")]
    ConnectionAborted,
    #[error("Connection failed")]
    ConnectionFailed,
    #[error("Name not resolved")]
    NameNotResolved,
    #[error("Internet disconnected")]
    InternetDisconnected,
    #[error("Socket not connected")]
    SocketNotConnected,
    #[error("SSL protocol error")]
    SslProtocolError,
    #[error("Address invalid")]
    AddressInvalid,
    #[error("Address unreachable")]
    AddressUnreachable,
    #[error("SSL client auth cert needed")]
    SslClientAuthCertNeeded,
    #[error("Tunnel connection failed")]
    TunnelConnectionFailed,
    #[error("SSL version or cipher mismatch")]
    SslVersionOrCipherMismatch,
    #[error("SSL renegotiation requested")]
    SslRenegotiationRequested,
    #[error("Proxy auth unsupported")]
    ProxyAuthUnsupported,
    #[error("Bad SSL client auth cert")]
    BadSslClientAuthCert,
    #[error("Connection timed out")]
    ConnectionTimedOut,
    #[error("Host resolver queue too large")]
    HostResolverQueueTooLarge,
    #[error("SOCKS connection failed")]
    SocksConnectionFailed,
    #[error("SOCKS connection host unreachable")]
    SocksConnectionHostUnreachable,
    #[error("ALPN negotiation failed")]
    AlpnNegotiationFailed,
    #[error("SSL no renegotiation")]
    SslNoRenegotiation,
    #[error("Winsock unexpected written bytes")]
    WinsockUnexpectedWrittenBytes,
    #[error("SSL decompression failure alert")]
    SslDecompressionFailureAlert,
    #[error("SSL bad record MAC alert")]
    SslBadRecordMacAlert,
    #[error("Proxy auth requested")]
    ProxyAuthRequested,
    #[error("Proxy connection failed")]
    ProxyConnectionFailed,
    #[error("Mandatory proxy configuration failed")]
    MandatoryProxyConfigurationFailed,
    #[error("Preconnect max socket limit")]
    PreconnectMaxSocketLimit,
    #[error("SSL client auth private key access denied")]
    SslClientAuthPrivateKeyAccessDenied,
    #[error("SSL client auth cert no private key")]
    SslClientAuthCertNoPrivateKey,
    #[error("Proxy certificate invalid")]
    ProxyCertificateInvalid,
    #[error("Name resolution failed")]
    NameResolutionFailed,
    #[error("Network access denied")]
    NetworkAccessDenied,
    #[error("Temporarily throttled")]
    TemporarilyThrottled,
    #[error("SSL client auth signature failed")]
    SslClientAuthSignatureFailed,
    #[error("Message too big")]
    MsgTooBig,
    #[error("WebSocket protocol error")]
    WsProtocolError,
    #[error("Address in use")]
    AddressInUse,
    #[error("SSL pinned key not in cert chain")]
    SslPinnedKeyNotInCertChain,
    #[error("Client auth cert type unsupported")]
    ClientAuthCertTypeUnsupported,
    #[error("SSL decrypt error alert")]
    SslDecryptErrorAlert,
    #[error("WebSocket throttle queue too large")]
    WsThrottleQueueTooLarge,
    #[error("SSL server cert changed")]
    SslServerCertChanged,
    #[error("SSL unrecognized name alert")]
    SslUnrecognizedNameAlert,
    #[error("Socket set receive buffer size error")]
    SocketSetReceiveBufferSizeError,
    #[error("Socket set send buffer size error")]
    SocketSetSendBufferSizeError,
    #[error("Socket receive buffer size unchangeable")]
    SocketReceiveBufferSizeUnchangeable,
    #[error("Socket send buffer size unchangeable")]
    SocketSendBufferSizeUnchangeable,
    #[error("SSL client auth cert bad format")]
    SslClientAuthCertBadFormat,
    #[error("ICANN name collision")]
    IcannNameCollision,
    #[error("SSL server cert bad format")]
    SslServerCertBadFormat,
    #[error("CT STH parsing failed")]
    CtSthParsingFailed,
    #[error("CT STH incomplete")]
    CtSthIncomplete,
    #[error("Unable to reuse connection for proxy auth")]
    UnableToReuseConnectionForProxyAuth,
    #[error("CT consistency proof parsing failed")]
    CtConsistencyProofParsingFailed,
    #[error("SSL obsolete cipher")]
    SslObsoleteCipher,
    #[error("WebSocket upgrade")]
    WsUpgrade,
    #[error("ReadIfReady not implemented")]
    ReadIfReadyNotImplemented,
    #[error("No buffer space")]
    NoBufferSpace,
    #[error("SSL client auth no common algorithms")]
    SslClientAuthNoCommonAlgorithms,
    #[error("Early data rejected")]
    EarlyDataRejected,
    #[error("Wrong version on early data")]
    WrongVersionOnEarlyData,
    #[error("TLS 1.3 downgrade detected")]
    Tls13DowngradeDetected,
    #[error("SSL key usage incompatible")]
    SslKeyUsageIncompatible,
    #[error("Invalid ECH config list")]
    InvalidEchConfigList,
    #[error("ECH not negotiated")]
    EchNotNegotiated,
    #[error("ECH fallback certificate invalid")]
    EchFallbackCertificateInvalid,
    #[error("Proxy unable to connect to destination")]
    ProxyUnableToConnectToDestination,
    #[error("Proxy delegate canceled connect request")]
    ProxyDelegateCanceledConnectRequest,
    #[error("Proxy delegate canceled connect response")]
    ProxyDelegateCanceledConnectResponse,

    // HTTP Errors
    #[error("Invalid URL")]
    InvalidUrl,
    #[error("Disallowed URL scheme")]
    DisallowedUrlScheme,
    #[error("Unknown URL scheme")]
    UnknownUrlScheme,
    #[error("Invalid redirect")]
    InvalidRedirect,
    #[error("Too many redirects")]
    TooManyRedirects,
    #[error("Redirect cycle detected")]
    RedirectCycleDetected,
    #[error("Content-Length mismatch")]
    ContentLengthMismatch,
    #[error("Socket closed by remote")]
    SocketRemoteClosed,
    #[error("Data received unexpectedly on idle socket")]
    DataReceivedUnexpectedly,
    #[error("Cookie prefix validation failed")]
    CookieInvalidPrefix,
    #[error("Cookie domain is a public suffix")]
    CookiePublicSuffix,
    #[error("Invalid HTTP header")]
    InvalidHeader,
    #[error("HTTP body read error")]
    HttpBodyError,
    #[error("Invalid UTF-8 in body")]
    InvalidUtf8,
    #[error("JSON parse error")]
    JsonParseError,
    #[error("Certificate pinning validation failed")]
    CertPinningFailed,
    #[error("Certificate transparency required but SCTs missing/invalid")]
    CertificateTransparencyRequired,
    #[error("Feature not implemented")]
    NotImplemented,
    #[error("File not found")]
    FileNotFound,
    #[error("Unsafe redirect")]
    UnsafeRedirect,
    #[error("Unsafe port")]
    UnsafePort,
    #[error("Invalid response")]
    InvalidResponse,
    #[error("Invalid chunked encoding")]
    InvalidChunkedEncoding,
    #[error("Method not supported")]
    MethodNotSupported,
    #[error("Unexpected proxy auth")]
    UnexpectedProxyAuth,
    #[error("Empty response")]
    EmptyResponse,
    #[error("Response headers too big")]
    ResponseHeadersTooBig,
    #[error("PAC script failed")]
    PacScriptFailed,
    #[error("Request range not satisfiable")]
    RequestRangeNotSatisfiable,
    #[error("Malformed identity")]
    MalformedIdentity,
    #[error("Content decoding failed")]
    ContentDecodingFailed,
    #[error("Network IO suspended")]
    NetworkIoSuspended,
    #[error("No supported proxies")]
    NoSupportedProxies,
    #[error("HTTP/2 protocol error")]
    Http2ProtocolError,
    #[error("Invalid auth credentials")]
    InvalidAuthCredentials,
    #[error("Unsupported auth scheme")]
    UnsupportedAuthScheme,
    #[error("Encoding detection failed")]
    EncodingDetectionFailed,
    #[error("Missing auth credentials")]
    MissingAuthCredentials,
    #[error("Unexpected security library status")]
    UnexpectedSecurityLibraryStatus,
    #[error("Misconfigured auth environment")]
    MisconfiguredAuthEnvironment,
    #[error("Undocumented security library status")]
    UndocumentedSecurityLibraryStatus,
    #[error("Response body too big to drain")]
    ResponseBodyTooBigToDrain,
    #[error("Response headers multiple Content-Length")]
    ResponseHeadersMultipleContentLength,
    #[error("Incomplete HTTP/2 headers")]
    IncompleteHttp2Headers,
    #[error("PAC not in DHCP")]
    PacNotInDhcp,
    #[error("Response headers multiple Content-Disposition")]
    ResponseHeadersMultipleContentDisposition,
    #[error("Response headers multiple Location")]
    ResponseHeadersMultipleLocation,
    #[error("HTTP/2 server refused stream")]
    Http2ServerRefusedStream,
    #[error("HTTP/2 PING failed")]
    Http2PingFailed,
    #[error("Incomplete chunked encoding")]
    IncompleteChunkedEncoding,
    #[error("QUIC protocol error")]
    QuicProtocolError,
    #[error("Response headers truncated")]
    ResponseHeadersTruncated,
    #[error("QUIC handshake failed")]
    QuicHandshakeFailed,
    #[error("HTTP/2 inadequate transport security")]
    Http2InadequateTransportSecurity,
    #[error("HTTP/2 flow control error")]
    Http2FlowControlError,
    #[error("HTTP/2 frame size error")]
    Http2FrameSizeError,
    #[error("HTTP/2 compression error")]
    Http2CompressionError,
    #[error("Proxy auth requested with no connection")]
    ProxyAuthRequestedWithNoConnection,
    #[error("HTTP/1.1 required")]
    Http11Required,
    #[error("Proxy HTTP/1.1 required")]
    ProxyHttp11Required,
    #[error("PAC script terminated")]
    PacScriptTerminated,
    #[error("Proxy required")]
    ProxyRequired,
    #[error("Invalid HTTP response")]
    InvalidHttpResponse,
    #[error("Content decoding init failed")]
    ContentDecodingInitFailed,
    #[error("HTTP/2 RST_STREAM NO_ERROR received")]
    Http2RstStreamNoErrorReceived,
    #[error("HTTP/2 pushed stream not available")]
    Http2PushedStreamNotAvailable,
    #[error("HTTP/2 claimed pushed stream reset by server")]
    Http2ClaimedPushedStreamResetByServer,
    #[error("Too many retries")]
    TooManyRetries,
    #[error("HTTP/2 stream closed")]
    Http2StreamClosed,
    #[error("HTTP/2 client refused stream")]
    Http2ClientRefusedStream,
    #[error("HTTP/2 pushed response does not match")]
    Http2PushedResponseDoesNotMatch,

    // Context-rich connection errors
    #[error("Connection to {host}:{port} failed")]
    ConnectionFailedTo {
        host: String,
        port: u16,
        #[source]
        source: Arc<io::Error>,
    },
    #[error("DNS resolution for {domain} failed")]
    NameNotResolvedFor {
        domain: String,
        #[source]
        source: Arc<io::Error>,
    },
    #[error("SSL handshake with {host} failed: {reason}")]
    SslHandshakeFailedWith { host: String, reason: String },

    // Cookie extraction errors (unified from CookieExtractionError)
    #[error("Browser {browser} not found")]
    BrowserNotFound { browser: String },
    #[error("Cookie database not found: {path}")]
    CookieDbNotFound { path: String },
    #[error("Failed to decrypt {browser} cookies: {reason}")]
    CookieDecryptionFailed { browser: String, reason: String },
    #[error("Cookie database locked (close browser)")]
    CookieDatabaseLocked,
    #[error("Browser version not supported: {version}")]
    CookieUnsupportedVersion { version: String },
    #[error("Platform not supported: {platform}")]
    CookiePlatformNotSupported { platform: String },
    #[error("Profile not found: {profile}")]
    CookieProfileNotFound { profile: String },
    #[error("Keyring unavailable")]
    CookieKeyringUnavailable,
    #[error("Invalid cookie data: {reason}")]
    CookieInvalidData { reason: String },
    #[error("Cookie database error: {message}")]
    CookieDatabaseError { message: String },

    #[error("Unknown error: {0}")]
    Unknown(i32),
}

impl NetError {
    pub fn as_i32(&self) -> i32 {
        match self {
            NetError::ConnectionClosed => -100,
            NetError::ConnectionReset => -101,
            NetError::ConnectionRefused => -102,
            NetError::ConnectionAborted => -103,
            NetError::ConnectionFailed => -104,
            NetError::NameNotResolved => -105,
            NetError::InternetDisconnected => -106,
            NetError::SocketNotConnected => -112,
            NetError::SslProtocolError => -107,
            NetError::AddressInvalid => -108,
            NetError::AddressUnreachable => -109,
            NetError::SslClientAuthCertNeeded => -110,
            NetError::TunnelConnectionFailed => -111,
            NetError::SslVersionOrCipherMismatch => -113,
            NetError::SslRenegotiationRequested => -114,
            NetError::ProxyAuthUnsupported => -115,
            NetError::BadSslClientAuthCert => -117,
            NetError::ConnectionTimedOut => -118,
            NetError::HostResolverQueueTooLarge => -119,
            NetError::SocksConnectionFailed => -120,
            NetError::SocksConnectionHostUnreachable => -121,
            NetError::AlpnNegotiationFailed => -122,
            NetError::SslNoRenegotiation => -123,
            NetError::WinsockUnexpectedWrittenBytes => -124,
            NetError::SslDecompressionFailureAlert => -125,
            NetError::SslBadRecordMacAlert => -126,
            NetError::ProxyAuthRequested => -127,
            NetError::ProxyConnectionFailed => -130,
            NetError::MandatoryProxyConfigurationFailed => -131,
            NetError::PreconnectMaxSocketLimit => -133,
            NetError::SslClientAuthPrivateKeyAccessDenied => -134,
            NetError::SslClientAuthCertNoPrivateKey => -135,
            NetError::ProxyCertificateInvalid => -136,
            NetError::NameResolutionFailed => -137,
            NetError::NetworkAccessDenied => -138,
            NetError::TemporarilyThrottled => -139,
            NetError::SslClientAuthSignatureFailed => -141,
            NetError::MsgTooBig => -142,
            NetError::WsProtocolError => -145,
            NetError::AddressInUse => -147,
            NetError::SslPinnedKeyNotInCertChain => -150,
            NetError::ClientAuthCertTypeUnsupported => -151,
            NetError::SslDecryptErrorAlert => -153,
            NetError::WsThrottleQueueTooLarge => -154,
            NetError::SslServerCertChanged => -156,
            NetError::SslUnrecognizedNameAlert => -159,
            NetError::SocketSetReceiveBufferSizeError => -160,
            NetError::SocketSetSendBufferSizeError => -161,
            NetError::SocketReceiveBufferSizeUnchangeable => -162,
            NetError::SocketSendBufferSizeUnchangeable => -163,
            NetError::SslClientAuthCertBadFormat => -164,
            NetError::IcannNameCollision => -166,
            NetError::SslServerCertBadFormat => -167,
            NetError::CtSthParsingFailed => -168,
            NetError::CtSthIncomplete => -169,
            NetError::UnableToReuseConnectionForProxyAuth => -170,
            NetError::CtConsistencyProofParsingFailed => -171,
            NetError::SslObsoleteCipher => -172,
            NetError::WsUpgrade => -173,
            NetError::ReadIfReadyNotImplemented => -174,
            NetError::NoBufferSpace => -176,
            NetError::SslClientAuthNoCommonAlgorithms => -177,
            NetError::EarlyDataRejected => -178,
            NetError::WrongVersionOnEarlyData => -179,
            NetError::Tls13DowngradeDetected => -180,
            NetError::SslKeyUsageIncompatible => -181,
            NetError::InvalidEchConfigList => -182,
            NetError::EchNotNegotiated => -183,
            NetError::EchFallbackCertificateInvalid => -184,
            NetError::ProxyUnableToConnectToDestination => -186,
            NetError::ProxyDelegateCanceledConnectRequest => -187,
            NetError::ProxyDelegateCanceledConnectResponse => -188,

            NetError::InvalidUrl => -300,
            NetError::DisallowedUrlScheme => -301,
            NetError::UnknownUrlScheme => -302,
            NetError::InvalidRedirect => -303,
            NetError::TooManyRedirects => -310,
            NetError::UnsafeRedirect => -311,
            NetError::UnsafePort => -312,
            NetError::InvalidResponse => -320,
            NetError::InvalidChunkedEncoding => -321,
            NetError::MethodNotSupported => -322,
            NetError::UnexpectedProxyAuth => -323,
            NetError::EmptyResponse => -324,
            NetError::ResponseHeadersTooBig => -325,
            NetError::PacScriptFailed => -327,
            NetError::RequestRangeNotSatisfiable => -328,
            NetError::MalformedIdentity => -329,
            NetError::ContentDecodingFailed => -330,
            NetError::NetworkIoSuspended => -331,
            NetError::NoSupportedProxies => -336,
            NetError::Http2ProtocolError => -337,
            NetError::InvalidAuthCredentials => -338,
            NetError::UnsupportedAuthScheme => -339,
            NetError::EncodingDetectionFailed => -340,
            NetError::MissingAuthCredentials => -341,
            NetError::UnexpectedSecurityLibraryStatus => -342,
            NetError::MisconfiguredAuthEnvironment => -343,
            NetError::UndocumentedSecurityLibraryStatus => -344,
            NetError::ResponseBodyTooBigToDrain => -345,
            NetError::ResponseHeadersMultipleContentLength => -346,
            NetError::IncompleteHttp2Headers => -347,
            NetError::PacNotInDhcp => -348,
            NetError::ResponseHeadersMultipleContentDisposition => -349,
            NetError::ResponseHeadersMultipleLocation => -350,
            NetError::Http2ServerRefusedStream => -351,
            NetError::Http2PingFailed => -352,
            NetError::ContentLengthMismatch => -354,
            NetError::IncompleteChunkedEncoding => -355,
            NetError::QuicProtocolError => -356,
            NetError::ResponseHeadersTruncated => -357,
            NetError::QuicHandshakeFailed => -358,
            NetError::Http2InadequateTransportSecurity => -360,
            NetError::Http2FlowControlError => -361,
            NetError::Http2FrameSizeError => -362,
            NetError::Http2CompressionError => -363,
            NetError::ProxyAuthRequestedWithNoConnection => -364,
            NetError::Http11Required => -365,
            NetError::ProxyHttp11Required => -366,
            NetError::PacScriptTerminated => -367,
            NetError::ProxyRequired => -368,
            NetError::InvalidHttpResponse => -370,
            NetError::ContentDecodingInitFailed => -371,
            NetError::Http2RstStreamNoErrorReceived => -372,
            NetError::Http2PushedStreamNotAvailable => -373,
            NetError::Http2ClaimedPushedStreamResetByServer => -374,
            NetError::TooManyRetries => -375,
            NetError::Http2StreamClosed => -376,
            NetError::Http2ClientRefusedStream => -377,
            NetError::Http2PushedResponseDoesNotMatch => -378,
            // Edge case errors (custom codes starting at -10000 to avoid collision with Chromium Blob errors)
            NetError::RedirectCycleDetected => -10000,
            NetError::SocketRemoteClosed => -10001,
            NetError::DataReceivedUnexpectedly => -10002,
            NetError::CookieInvalidPrefix => -10003,
            NetError::CookiePublicSuffix => -10004,
            NetError::InvalidHeader => -10005,
            NetError::HttpBodyError => -10006,
            NetError::InvalidUtf8 => -10007,
            NetError::JsonParseError => -10008,
            NetError::CertPinningFailed => -10009,
            NetError::CertificateTransparencyRequired => -10010,
            NetError::NotImplemented => -10011,
            NetError::FileNotFound => -10012,
            // Context variants (same code as simple variant)
            NetError::ConnectionFailedTo { .. } => -104,
            NetError::NameNotResolvedFor { .. } => -105,
            NetError::SslHandshakeFailedWith { .. } => -107,
            // Cookie extraction errors
            NetError::BrowserNotFound { .. } => -10020,
            NetError::CookieDbNotFound { .. } => -10021,
            NetError::CookieDecryptionFailed { .. } => -10022,
            NetError::CookieDatabaseLocked => -10023,
            NetError::CookieUnsupportedVersion { .. } => -10024,
            NetError::CookiePlatformNotSupported { .. } => -10025,
            NetError::CookieProfileNotFound { .. } => -10026,
            NetError::CookieKeyringUnavailable => -10027,
            NetError::CookieInvalidData { .. } => -10028,
            NetError::CookieDatabaseError { .. } => -10029,
            NetError::Unknown(code) => *code,
        }
    }

    // Helper constructors for context-rich errors

    /// Create connection failed error with context.
    pub fn connection_failed_to(host: impl Into<String>, port: u16, source: io::Error) -> Self {
        Self::ConnectionFailedTo {
            host: host.into(),
            port,
            source: Arc::new(source),
        }
    }

    /// Create DNS resolution error with context.
    pub fn dns_failed(domain: impl Into<String>, source: io::Error) -> Self {
        Self::NameNotResolvedFor {
            domain: domain.into(),
            source: Arc::new(source),
        }
    }

    /// Create SSL handshake error with context.
    pub fn ssl_handshake_failed(host: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::SslHandshakeFailedWith {
            host: host.into(),
            reason: reason.into(),
        }
    }

    /// Create browser not found error.
    pub fn browser_not_found(browser: impl Into<String>) -> Self {
        Self::BrowserNotFound {
            browser: browser.into(),
        }
    }

    /// Create cookie decryption failed error.
    pub fn cookie_decryption_failed(browser: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::CookieDecryptionFailed {
            browser: browser.into(),
            reason: reason.into(),
        }
    }

    /// Create cookie database not found error.
    pub fn cookie_db_not_found(path: impl Into<String>) -> Self {
        Self::CookieDbNotFound { path: path.into() }
    }

    /// Create cookie invalid data error.
    pub fn cookie_invalid_data(reason: impl Into<String>) -> Self {
        Self::CookieInvalidData {
            reason: reason.into(),
        }
    }
}

impl From<io::Error> for NetError {
    fn from(e: io::Error) -> Self {
        use io::ErrorKind;
        match e.kind() {
            ErrorKind::ConnectionRefused => Self::ConnectionRefused,
            ErrorKind::ConnectionReset => Self::ConnectionReset,
            ErrorKind::ConnectionAborted => Self::ConnectionAborted,
            ErrorKind::NotConnected => Self::SocketNotConnected,
            ErrorKind::AddrInUse => Self::AddressInUse,
            ErrorKind::AddrNotAvailable => Self::AddressUnreachable,
            ErrorKind::TimedOut => Self::ConnectionTimedOut,
            _ => Self::ConnectionFailed,
        }
    }
}

impl From<url::ParseError> for NetError {
    fn from(_: url::ParseError) -> Self {
        Self::InvalidUrl
    }
}

impl From<i32> for NetError {
    fn from(code: i32) -> Self {
        match code {
            -100 => NetError::ConnectionClosed,
            -101 => NetError::ConnectionReset,
            -102 => NetError::ConnectionRefused,
            -103 => NetError::ConnectionAborted,
            -104 => NetError::ConnectionFailed,
            -105 => NetError::NameNotResolved,
            -106 => NetError::InternetDisconnected,
            -112 => NetError::SocketNotConnected,
            -107 => NetError::SslProtocolError,
            -108 => NetError::AddressInvalid,
            -109 => NetError::AddressUnreachable,
            -110 => NetError::SslClientAuthCertNeeded,
            -111 => NetError::TunnelConnectionFailed,
            -113 => NetError::SslVersionOrCipherMismatch,
            -114 => NetError::SslRenegotiationRequested,
            -115 => NetError::ProxyAuthUnsupported,
            -117 => NetError::BadSslClientAuthCert,
            -118 => NetError::ConnectionTimedOut,
            -119 => NetError::HostResolverQueueTooLarge,
            -120 => NetError::SocksConnectionFailed,
            -121 => NetError::SocksConnectionHostUnreachable,
            -122 => NetError::AlpnNegotiationFailed,
            -123 => NetError::SslNoRenegotiation,
            -124 => NetError::WinsockUnexpectedWrittenBytes,
            -125 => NetError::SslDecompressionFailureAlert,
            -126 => NetError::SslBadRecordMacAlert,
            -127 => NetError::ProxyAuthRequested,
            -130 => NetError::ProxyConnectionFailed,
            -131 => NetError::MandatoryProxyConfigurationFailed,
            -133 => NetError::PreconnectMaxSocketLimit,
            -134 => NetError::SslClientAuthPrivateKeyAccessDenied,
            -135 => NetError::SslClientAuthCertNoPrivateKey,
            -136 => NetError::ProxyCertificateInvalid,
            -137 => NetError::NameResolutionFailed,
            -138 => NetError::NetworkAccessDenied,
            -139 => NetError::TemporarilyThrottled,
            -141 => NetError::SslClientAuthSignatureFailed,
            -142 => NetError::MsgTooBig,
            -145 => NetError::WsProtocolError,
            -147 => NetError::AddressInUse,
            -150 => NetError::SslPinnedKeyNotInCertChain,
            -151 => NetError::ClientAuthCertTypeUnsupported,
            -153 => NetError::SslDecryptErrorAlert,
            -154 => NetError::WsThrottleQueueTooLarge,
            -156 => NetError::SslServerCertChanged,
            -159 => NetError::SslUnrecognizedNameAlert,
            -160 => NetError::SocketSetReceiveBufferSizeError,
            -161 => NetError::SocketSetSendBufferSizeError,
            -162 => NetError::SocketReceiveBufferSizeUnchangeable,
            -163 => NetError::SocketSendBufferSizeUnchangeable,
            -164 => NetError::SslClientAuthCertBadFormat,
            -166 => NetError::IcannNameCollision,
            -167 => NetError::SslServerCertBadFormat,
            -168 => NetError::CtSthParsingFailed,
            -169 => NetError::CtSthIncomplete,
            -170 => NetError::UnableToReuseConnectionForProxyAuth,
            -171 => NetError::CtConsistencyProofParsingFailed,
            -172 => NetError::SslObsoleteCipher,
            -173 => NetError::WsUpgrade,
            -174 => NetError::ReadIfReadyNotImplemented,
            -176 => NetError::NoBufferSpace,
            -177 => NetError::SslClientAuthNoCommonAlgorithms,
            -178 => NetError::EarlyDataRejected,
            -179 => NetError::WrongVersionOnEarlyData,
            -180 => NetError::Tls13DowngradeDetected,
            -181 => NetError::SslKeyUsageIncompatible,
            -182 => NetError::InvalidEchConfigList,
            -183 => NetError::EchNotNegotiated,
            -184 => NetError::EchFallbackCertificateInvalid,
            -186 => NetError::ProxyUnableToConnectToDestination,
            -187 => NetError::ProxyDelegateCanceledConnectRequest,
            -188 => NetError::ProxyDelegateCanceledConnectResponse,

            -300 => NetError::InvalidUrl,
            -301 => NetError::DisallowedUrlScheme,
            -302 => NetError::UnknownUrlScheme,
            -303 => NetError::InvalidRedirect,
            -310 => NetError::TooManyRedirects,
            -311 => NetError::UnsafeRedirect,
            -312 => NetError::UnsafePort,
            -320 => NetError::InvalidResponse,
            -321 => NetError::InvalidChunkedEncoding,
            -322 => NetError::MethodNotSupported,
            -323 => NetError::UnexpectedProxyAuth,
            -324 => NetError::EmptyResponse,
            -325 => NetError::ResponseHeadersTooBig,
            -327 => NetError::PacScriptFailed,
            -328 => NetError::RequestRangeNotSatisfiable,
            -329 => NetError::MalformedIdentity,
            -330 => NetError::ContentDecodingFailed,
            -331 => NetError::NetworkIoSuspended,
            -336 => NetError::NoSupportedProxies,
            -337 => NetError::Http2ProtocolError,
            -338 => NetError::InvalidAuthCredentials,
            -339 => NetError::UnsupportedAuthScheme,
            -340 => NetError::EncodingDetectionFailed,
            -341 => NetError::MissingAuthCredentials,
            -342 => NetError::UnexpectedSecurityLibraryStatus,
            -343 => NetError::MisconfiguredAuthEnvironment,
            -344 => NetError::UndocumentedSecurityLibraryStatus,
            -345 => NetError::ResponseBodyTooBigToDrain,
            -346 => NetError::ResponseHeadersMultipleContentLength,
            -347 => NetError::IncompleteHttp2Headers,
            -348 => NetError::PacNotInDhcp,
            -349 => NetError::ResponseHeadersMultipleContentDisposition,
            -350 => NetError::ResponseHeadersMultipleLocation,
            -351 => NetError::Http2ServerRefusedStream,
            -352 => NetError::Http2PingFailed,
            -354 => NetError::ContentLengthMismatch,
            -355 => NetError::IncompleteChunkedEncoding,
            -356 => NetError::QuicProtocolError,
            -357 => NetError::ResponseHeadersTruncated,
            -358 => NetError::QuicHandshakeFailed,
            -360 => NetError::Http2InadequateTransportSecurity,
            -361 => NetError::Http2FlowControlError,
            -362 => NetError::Http2FrameSizeError,
            -363 => NetError::Http2CompressionError,
            -364 => NetError::ProxyAuthRequestedWithNoConnection,
            -365 => NetError::Http11Required,
            -366 => NetError::ProxyHttp11Required,
            -367 => NetError::PacScriptTerminated,
            -368 => NetError::ProxyRequired,
            -370 => NetError::InvalidHttpResponse,
            -371 => NetError::ContentDecodingInitFailed,
            -372 => NetError::Http2RstStreamNoErrorReceived,
            -373 => NetError::Http2PushedStreamNotAvailable,
            -374 => NetError::Http2ClaimedPushedStreamResetByServer,
            -375 => NetError::TooManyRetries,
            -376 => NetError::Http2StreamClosed,
            -377 => NetError::Http2ClientRefusedStream,
            -378 => NetError::Http2PushedResponseDoesNotMatch,
            // Fix critical error code collisions in neterror.rs where custom chromenet errors
            // overlap with Chromium's Blob error range (-900 to -906), and update loadstate.rs
            // to include missing load states found in Chromium's load_states_list.h.
            //
            // User Review Required
            // WARNING
            //
            // Breaking Change: NetError codes for custom errors (like RedirectCycleDetected)
            // will change from the -900 range to the -10000 range to avoid future conflicts
            // with Chromium's Blob errors.
            -10000 => NetError::RedirectCycleDetected,
            -10001 => NetError::SocketRemoteClosed,
            -10002 => NetError::DataReceivedUnexpectedly,
            -10003 => NetError::CookieInvalidPrefix,
            -10004 => NetError::CookiePublicSuffix,
            -10005 => NetError::InvalidHeader,
            -10006 => NetError::HttpBodyError,
            -10007 => NetError::InvalidUtf8,
            -10008 => NetError::JsonParseError,
            -10009 => NetError::CertPinningFailed,
            -10010 => NetError::NotImplemented,
            -10011 => NetError::FileNotFound,
            _ => NetError::Unknown(code),
        }
    }
}
