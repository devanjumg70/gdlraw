# Development Guidelines - `chromenet`

This document outlines the strict project rules, naming conventions, and architectural principles for the `chromenet` library. These rules must be followed for all future development to ensure consistency with the "Clean & Raw" philosophy.

## 1. Naming Conventions ("The Clean Style")
We enforce a strict "no underscore" policy for file and module names to maintain a clean, unified aesthetic.

*   **Crates**: `chromenet` (Not `chrome_net`)
*   **Modules**: `urlrequest`, `connectjob`, `streamfactory`.
    *   **Do**: `src/urlrequest/mod.rs`
    *   **Don't**: `src/url_request/mod.rs`
*   **Files**: `neterror.rs`, `loadstate.rs`.
    *   **Do**: `connectjob.rs`
    *   **Don't**: `connect_job.rs`
*   **Structs/Enums**: PascalCase (Standard Rust).
    *   e.g., `ClientSocketPool`, `HttpNetworkTransaction`.

## 2. Architectural Principles ("The Raw Strategy")
This library is a **raw port** of Chromium's C++ network stack (`//net`).

### Rule #1: No High-Level Clients
*   **Forbid**: `reqwest`, `wreq`, `ureq`, `surf`.
*   **Allow**: `tokio` (Runtime), `hyper` (Low-level framing/parsing ONLY).

### Rule #2: BoringSSL is Mandatory
*   We must use `boring` (Rust bindings to Google's BoringSSL) for all cryptography to match Chromium's TLS footprint exactly.
*   **Do not use**: `rustls`, `native-tls`, `openssl` (unless mocking).

### Rule #3: Strict Chromium Mapping
We implement logic by mirroring Chromium's classes, not by "inventing" new Rust abstractions.

| Chromium (C++) | Rust (`chromenet`) | Responsibility |
| :--- | :--- | :--- |
| `net::ClientSocketPool` | `socket/pool.rs` | Enforce limits (6 conn/host). |
| `net::HttpNetworkTransaction` | `http/transaction.rs` | State machine driver. |
| `net::ConnectJob` | `socket/connectjob.rs` | DNS -> TCP -> SSL. |
| `net::URLRequest` | `urlrequest/request.rs` | Public Facade. |

## 3. Implementation Details

### Connection Pooling
*   **Constraint**: You MUST check connection limits before connecting.
*   **Limits**: Max 6 connections per host (group), 256 total.
*   **Concurrency**: Use `DashMap` for internal state to handle multi-threaded requests safely.

### State Machines
*   Use Rust `enum` variants to represent C++ state constants (e.g., `STATE_SEND_REQUEST` -> `State::SendRequest`).
*   Transitions should be driven by an async `do_loop` method.

## 4. Code Quality
*   **Formatting**: Run `cargo fmt` before every commit.
*   **Linting**: Run `cargo clippy`; warnings should be treated as errors.
*   **Testing**:
    *   Unit tests for logic (pools, transaction states).
    *   Integration tests (`tests/`) for real network I/O.
