# Chromenet Capabilities Documentation

This directory contains comprehensive documentation about chromenet's features, limitations, and production readiness.

## Contents

| Document | Description |
|----------|-------------|
| [features.md](features.md) | Complete feature list with API entry points |
| [limitations.md](limitations.md) | Known limitations and unsupported use cases |
| [production_status.md](production_status.md) | Production readiness assessment |

## Quick Summary

**chromenet** is a Chromium-inspired HTTP client library for Rust featuring:
- 🌐 HTTP/1.1 & HTTP/2 with connection pooling
- 🔒 BoringSSL-based TLS with fingerprint control
- 🍪 RFC 6265 compliant cookie management
- 🎭 67 browser emulation profiles
- 🔐 HSTS, certificate pinning, CT verification
- 📡 WebSocket, multipart uploads, proxy support

**Status**: ✅ Production-ready for HTTP/HTTPS workloads
