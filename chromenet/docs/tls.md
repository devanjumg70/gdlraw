# TLS Module

Security features for TLS connections, including HSTS, certificate pinning, and CT verification.

## Files
| File | Lines | Purpose |
|------|-------|---------|
| [hsts.rs](../src/tls/hsts.rs) | ~331 | HTTP Strict Transport Security |
| [pinning.rs](../src/tls/pinning.rs) | ~280 | Certificate pinning |
| [ct.rs](../src/tls/ct.rs) | ~77 | CT types and SCT structures |
| [ctverifier.rs](../src/tls/ctverifier.rs) | ~403 | Multi-log CT verification |

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
| **Zero-Alloc Check** | ✅ Optimized parent domain iteration |

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

## Certificate Transparency

Verify Signed Certificate Timestamps (SCTs) against known CT logs.

> [!NOTE]
> Full infrastructure implemented in `MultiLogCtVerifier` (403 lines). Signature verification uses a simplified implementation - all non-empty signatures from known logs are accepted.

### Features
| Feature | Status |
|---------|--------|
| SCT list parsing | ✅ RFC 6962 compliant |
| Log registry | ✅ Add/lookup logs by ID |
| Timestamp validation | ✅ Reject future timestamps |
| Requirement levels | ✅ NotRequired, SoftFail, Required |
| ECDSA verification | ⚠️ Simplified (placeholder) |

### Usage
```rust
use chromenet::tls::ctverifier::{MultiLogCtVerifier, CtLog};
use chromenet::tls::ct::CtRequirement;

let verifier = MultiLogCtVerifier::new()
    .with_requirement(CtRequirement::SoftFail);

// Add known CT logs
verifier.add_log(CtLog::new(log_id, public_key, "Google Argon"));

// Verify SCTs from certificate
let results = verifier.verify(&scts, &cert_der, current_time);
verifier.check_requirements(&results)?;
```

### Requirement Levels
| Level | Behavior |
|-------|----------|
| `NotRequired` | CT not checked |
| `SoftFail` | Log warning if missing/invalid |
| `Required` | Block connection without valid SCTs |

---

## Chromium Mapping

| Chromium C++ | Rust | Purpose |
|--------------|------|---------|
| `TransportSecurityState` | `HstsStore` | HSTS enforcement |
| `TransportSecurityState::PKPState` | `PinStore` | Certificate pinning |
| `MultiLogCTVerifier` | `MultiLogCtVerifier` | SCT verification |
