# Changelog

All notable changes to this project will be documented in this file.

## [0.1.0] - 2025-12-28

### Added
- **Browser Emulation**: Full support for Chrome, Firefox, Safari, Edge, OkHttp, and Opera profiles (67 total variants).
- **TLS Fingerprinting**: Integration with BoringSSL for 30+ TLS options including JA3/JA4 parameters.
- **HTTP/2 Fingerprinting**: Support for SETTINGS order, pseudo-header order, and priority frames.
- **Connection Pooling**: Chromium-style connection limits (6 per host, 256 total) via `ClientSocketPool`.
- **Advanced Proxy Support**: HTTP, HTTPS (TLS-in-TLS), and SOCKS5 proxy support with rotation and authentication.
- **Cookie Management**: RFC 6265 compliant `CookieMonster` with specialized browser extraction (Chrome, Firefox, Safari on Linux/macOS/Windows).
- **HSTS Persistence**: Preload list support and dynamic HSTS state persistence via JSON.
- **Public Key Pinning**: SPKI hash verification for certificate pinning.
- **Header Management**: `orderedheaders` crate integration for preserving header order and casing.
- **WebSocket Client**: Fully featured WebSocket client (`ws` module) based on `tokio-tungstenite`.
- **HTTP/3 Readiness**: Type definitions and structure for QUIC integration (`quic` module).
- **Multipart Uploads**: Ergonomic API for multipart/form-data requests.
- **Streaming Bodies**: `BodyStream` for memory-efficient handling of large response bodies.
- **Testing Suite**: Comprehensive unit, integration, and fuzz-style tests (230+ tests) covering all core modules.

### Changed
- Refactored `Client` and `ClientBuilder` for better ergonomics and builder pattern consistency.
- Unified TLS configuration to support both emulation-driven and manual settings.
- Optimized `HttpNetworkTransaction` state machine for robustness and correct error handling.

### Fixed
- Fixed hanging tests in socket pool concurrency limits.
- Resolved all Clippy warnings and formatting issues.
- Corrected import paths and visibility modifiers across the crate.

### Security
- Mandated use of `boring` (BoringSSL/OpenSSL) for FIPS compliance and accurate browser TLS behavior.
- Implemented constant-time comparison for sensitive tokens where applicable.
