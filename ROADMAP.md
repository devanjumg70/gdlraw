# `chromenet` Development Roadmap

This document serves as the high-level guide for evolving `chromenet` from a basic raw HTTP client into a sophisticated, production-grade network stack capable of advanced impersonation (similar to `yt-dlp` but project-agnostic).

## Core Philosophy
*   **Raw & Custom**: We build on top of `boring` (BoringSSL) and sockets. No high-level wrappers.
*   **Chromium-Aligned**: We conform to Chromium's architectural patterns (`//net`).
*   **Impersonation-First**: Every layer (TLS, Headers, HTTP/2) must be configurable to mimic real browsers.

---

## Phase 1: Foundation (Completed) ✅
*   [x] **Structure**: `crate` scaffolding, module layout (`socket`, `http`, `urlrequest`).
*   [x] **Transport**: Basic `ClientSocketPool` (strict limits) and `ConnectJob` (DNS->TCP->SSL).
*   [x] **Transaction**: Basic `HttpNetworkTransaction` state machine.
*   [x] **Public API**: Basic `URLRequest`.

---

## Phase 2: The Identity Layer (Impersonation) 🚧
**Goal**: Make `chromenet` indistinguishable from real browsers (Chrome, Firefox, Safari).

### 1. TLS Fingerprinting (`socket/connectjob.rs`)
*   **Current Specification**: Uses default BoringSSL config.
*   **Objective**: Allow configuring the TLS Client Hello.
*   **Tasks**:
    *   Implement `TlsConfig` struct to specify cipher suites, extensions, and curves.
    *   Create pre-sets for Chrome, Firefox, Safari.
    *   **Chromium Ref**: `net::SSLClientSocketImpl`, `net::SSLConfig`.

### 2. Header Order & Management (`http/transaction.rs`)
*   **Current Specification**: Hardcoded `Host` header.
*   **Objective**: Full control over header casing and ordering (crucial for fingerprinting).
*   **Tasks**:
    *   Implement `HeaderMap` wrapper that preserves order (standard `http::HeaderMap` does not guarantee order preservation in the same way browsers do).
    *   Implement "Default Headers" injection based on User-Agent (Accept, Accept-Language, sec-ch-ua).
    *   **Chromium Ref**: `net::HttpNetworkTransaction::BuildRequestHeaders`.

### 3. Device Emulation (`urlrequest/device.rs`)
*   **Objective**: Easy API to "become" a device.
*   **Tasks**:
    *   Port the `all_devices.json` data we extracted into a registry.
    *   `URLRequest::set_device("Pixel 7")` automatically configures Headers + Client Hints + User-Agent.

---

## Phase 3: State Management (Cookies & Persistence)
**Goal**: Robust session handling for complex logins and flows.

### 1. Cookie Jar (`base/cookie.rs`)
*   **Current Specification**: None.
*   **Objective**: Full RFC 6265 cookie support.
*   **Tasks**:
    *   Implement or integrate a `CookieStore` (mimicking `net::CookieMonster`).
    *   Hooks in `HttpNetworkTransaction` to `LoadCookies` before sending and `SaveCookies` after reading headers.
    *   **Chromium Ref**: `net::CookieMonster`, `net::URLRequestHttpJob`.

### 2. Redirect Handling
*   **Current Specification**: None.
*   **Objective**: Handle 3xx status codes automatically.
*   **Tasks**:
    *   Update `HttpNetworkTransaction` to detect redirects.
    *   Implement `RestartTransaction` logic to follow the new URL.

---

## Phase 4: Advanced Transport (Network Resilience)
**Goal**: Handle hostile network conditions and scale.

### 1. Proxy Support (`socket/proxy.rs`)
*   **Objective**: Support HTTP/HTTPS/SOCKS proxies.
*   **Tasks**:
    *   Modify `ConnectJob` to tunnel through proxies.
    *   Handle Proxy Auth (407).
    *   **Chromium Ref**: `net::ProxyResolutionService`, `net::ProxyClientSocket`.

### 2. HTTP/2 & HTTP/3
*   **Objective**: Modern protocol support.
*   **Tasks**:
    *   Upgrade `HttpStreamFactory` to negotiate ALPN.
    *   Integrate `h2` crate with `boring` stream.

### 3. Reliability (Retries)
*   **Objective**: Automatic recovery from network blips.
*   **Tasks**:
    *   Implement Chromium's "Retry on Connection Failure" logic in `HttpNetworkTransaction`.
    *   Exponential backoff.

---

## Recommended Next Steps
1.  **Header Ordering & Management**: This is the easiest "high impact" change for impersonation.
2.  **TLS Fingerprinting**: The most critical feature for evading bot detection.
3.  **Cookies**: Essential for any login-based scraping.
