# Master Plan: Unified Chromenet & Wreq

## Objective

Merge the high-level features and usability of **wreq** into the robust, Chromium-architected core of **chromenet** to create a single, powerful HTTP client crate.

## Philosophy

- **chromenet** provides **the engine**: SocketPool, ConnectJob, URLRequestContext, CookieMonster
- **wreq** provides **the steering wheel**: Client API, Request Builders, Body handling, and specialized modules (Cookies, Proxy, Multipart)

---

## Phase 1: Core Networking & TLS (Partially Complete)

**Goal**: Ensure chromenet has the low-level capabilities to match wreq's emulation.

| Task | Status | Notes |
|------|--------|-------|
| TLS Options | ✅ Completed | Ported to `socket/tls/options.rs` with builder pattern |
| Impersonation | ✅ Completed | 8 targets (Chrome124/128, Firefox128/129, Safari17/18, OkHttp4/5) in `socket/tls/impersonate.rs` |
| DNS | ⚠️ Pending | Evaluate wreq/src/dns vs chromenet's host resolver. Merge DoH features |

---

## Phase 2: HttpStream & Connection Layer

**Goal**: Align connection handling and HTTP versions.

| Task | Status | Notes |
|------|--------|-------|
| HTTP/2 Settings | ⚠️ Verify | Verify chromenet's H2 settings match wreq's emulation capabilities |
| Stream Factory | ⚠️ Verify | Ensure HttpStreamFactory handles HTTP/1.0, 1.1, 2 robustly |

---

## Phase 3: Proxy & Tunneling

**Goal**: Integrate wreq's advanced proxy support into chromenet.

| Task | Status | Notes |
|------|--------|-------|
| Proxy Resolution | ❌ Missing | Port wreq/src/proxy/matcher.rs for NO_PROXY, domain matching |
| System Proxies | ❌ Missing | Port wreq/src/proxy/{mac,win}.rs for automatic system proxy detection |
| Authentication | ⚠️ Compare | Enhance chromenet's proxy auth with wreq's logic if superior |

---

## Phase 4: Cookie Management

**Goal**: Unified Cookie Store.

| Task | Status | Notes |
|------|--------|-------|
| Cookie Store | ✅ Robust | chromenet's CookieMonster is Chromium-ported; check wreq for API conveniences |
| Persistence | ✅ Complete | chromenet has persistence.rs |

---

## Phase 5: Client API & Request Building

**Goal**: A user-friendly, high-level API aka "The Wreq Experience".

| Task | Status | Notes |
|------|--------|-------|
| Client Struct | ❌ Missing | Create high-level Client wrapping URLRequestContext, mimicking wreq::Client |
| Request Builder | ❌ Missing | Port wreq/src/client/request.rs to construct URLRequest objects |
| Response | ❌ Missing | Port wreq/src/client/response.rs and body.rs for easy body access |
| Middleware/Layers | ⚠️ Evaluate | Evaluate wreq/src/client/layer for interceptor/middleware system |

---

## Phase 6: Advanced Features

**Goal**: Feature parity.

| Task | Status | Notes |
|------|--------|-------|
| Multipart | ❌ Missing | Port wreq/src/client/multipart.rs |
| WebSockets | ❌ Missing | Port wreq/src/client/ws if chromenet lacks WS |
| Redirects | ⚠️ Verify | Ensure URLRequestJob handles redirects as flexibly as wreq/src/redirect.rs |

---

## Phase 7: Cleanup & Unification

**Goal**: Single crate, clean exports.

| Task | Status | Notes |
|------|--------|-------|
| Exports | ⚠️ Pending | Re-export core types from the root |
| Documentation | ⚠️ Pending | Unified docs |
| Tests | ⚠️ Pending | Port key integration tests from wreq to verify chromenet |

---

## Immediate Next Step Recommendation

**Start with Phase 5 (Client API)** to make chromenet usable immediately with the new TLS features, then fill in Proxy/Cookie gaps.

---

## Repository Structure

```
/home/ubuntu/projects/
├── gdlraw/chromenet/     # Main chromenet crate (Chromium-style)
├── wreq/                 # Reference wreq crate
├── wreq-util/            # Emulation profiles (70+ browser targets)
├── http2/                # Fork of h2 crate for HTTP/2
└── chromium/             # Chromium source reference
```
