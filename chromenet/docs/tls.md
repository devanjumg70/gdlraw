# TLS Module

Security features for TLS connections, including HSTS, certificate pinning, and CT verification.

## Files
| File | Lines | Purpose |
|------|-------|---------|
| [hsts.rs](../src/tls/hsts.rs) | ~220 | HTTP Strict Transport Security |
| [pinning.rs](../src/tls/pinning.rs) | ~260 | Certificate pinning |
| [ct.rs](../src/tls/ct.rs) | ~125 | Certificate Transparency |

---

## HSTS (HTTP Strict Transport Security)

Force HTTPS for domains with HSTS policies.

### Features
| Feature | Status |
|---------|--------|
| Preloaded domains | ✅ Google, GitHub, PayPal, etc. |
| Dynamic headers | ✅ Parse Strict-Transport-Security |
| Subdomain matching | ✅ includeSubDomains support |
| Expiration | ✅ max-age handling |

### Usage
```rust
use chromenet::tls::hsts::HstsStore;

// Create with preloaded domains
let hsts = HstsStore::with_preload();

// Check if HTTPS upgrade needed
if hsts.should_upgrade("mail.google.com") {
    url.set_scheme("https");
}

// Add from response header
hsts.add_from_header("example.com", "max-age=31536000; includeSubDomains");
```

### Preloaded Domains
- google.com (+ subdomains)
- github.com (+ subdomains)
- paypal.com (+ subdomains)
- facebook.com (+ subdomains)
- twitter.com (+ subdomains)

---

## Certificate Pinning

Verify server certificates match expected SPKI hashes.

### Features
| Feature | Status |
|---------|--------|
| SPKI hash verification | ✅ SHA-256 |
| Subdomain pinning | ✅ Optional |
| Expiration | ✅ Fail-open when expired |
| Multiple pins | ✅ Any match succeeds |

### Usage
```rust
use chromenet::tls::pinning::{PinStore, PinSet};

let store = PinStore::new();

// Add pins for a domain
let mut pins = PinSet::new("example.com")
    .include_subdomains(true);
pins.add_pin([0x42; 32]); // SHA-256 of SPKI
store.add(pins);

// Verify certificate chain
let cert_hashes = get_cert_chain_hashes(&certs);
store.check("api.example.com", &cert_hashes)?;
```

### Security Model
- **Fail-open on expiry**: Expired pins don't block connections
- **Any match**: Connection allowed if ANY pin matches
- **Subdomain inheritance**: Optional via `include_subdomains(true)`

---

## Certificate Transparency (Stub)

Verify Signed Certificate Timestamps (SCTs).

> **Note**: This is a stub implementation. Full CT requires log public keys and signature verification.

### Usage
```rust
use chromenet::tls::ct::{CtVerifier, CtRequirement};

let verifier = CtVerifier::new()
    .with_requirement(CtRequirement::SoftFail);

// Soft-fail: log warning but allow connection
verifier.verify(&cert_chain, &scts)?;
```

### Requirement Levels
| Level | Behavior |
|-------|----------|
| `NotRequired` | CT not checked |
| `SoftFail` | Log warning if missing |
| `Required` | Block if no valid SCTs |

---

## Chromium Mapping

| Chromium C++ | Rust | Purpose |
|--------------|------|---------|
| `TransportSecurityState` | `HstsStore` | HSTS enforcement |
| `TransportSecurityState::PKPState` | `PinStore` | Certificate pinning |
| `CTVerifier` | `CtVerifier` | SCT verification |
