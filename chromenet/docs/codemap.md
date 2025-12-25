# Chromenet Codemap

Complete flow diagrams and scenarios for the chromenet library.

**Stats:** ~6,000 LOC | 112 tests | 9 benchmarks | 4 examples

## High-Level Architecture

```mermaid
graph TB
    subgraph "Public API"
        URL[URLRequest]
        CTX[URLRequestContext]
    end

    subgraph "HTTP Layer"
        TXN[HttpNetworkTransaction]
        SF[HttpStreamFactory]
        H2C[H2SessionCache]
        RETRY[Retry Logic]
    end

    subgraph "Connection Layer"
        POOL[ClientSocketPool]
        CJ[ConnectJob]
        AUTH[AuthCache]
    end

    subgraph "Transport Layer"
        TCP[TcpStream]
        TLS[SslStream]
        PROXY[Proxy Handshake]
    end

    subgraph "Security Layer"
        HSTS[HstsStore]
        PINS[PinStore]
        CT[CtVerifier]
    end

    subgraph "Cookie Management"
        COOKIE[CookieMonster]
        PSL[PSL Validation]
        BROWSER[Browser Extraction]
        PERSIST[Persistence]
    end

    URL --> CTX
    CTX --> TXN
    TXN --> RETRY
    TXN --> SF
    SF --> H2C
    SF --> POOL
    POOL --> CJ
    POOL --> AUTH
    CJ --> TCP
    CJ --> PROXY
    PROXY --> TLS
    TCP --> TLS

    TXN --> COOKIE
    COOKIE --> PSL
    COOKIE --> PERSIST
    BROWSER --> COOKIE
    
    CJ --> HSTS
    CJ --> PINS
```

---

## Scenario 1: Simple HTTPS Request

```mermaid
sequenceDiagram
    participant App
    participant URLRequest
    participant HSTS
    participant Transaction
    participant Pool
    participant ConnectJob
    participant Server

    App->>URLRequest: new(https://example.com)
    URLRequest->>HSTS: should_upgrade?
    HSTS-->>URLRequest: No (already HTTPS)
    URLRequest->>Transaction: start()
    Transaction->>Pool: request_socket()
    Pool->>ConnectJob: connect()
    ConnectJob->>Server: DNS lookup
    ConnectJob->>Server: TCP connect (Happy Eyeballs)
    ConnectJob->>Server: TLS handshake
    ConnectJob-->>Pool: SslStream
    Pool-->>Transaction: HttpStream
    Transaction->>Server: HTTP/2 request
    Server-->>Transaction: Response + Set-Cookie
    Transaction->>CookieMonster: save cookies (PSL validated)
    Transaction-->>App: Response
```

---

## Scenario 2: HSTS Upgrade

```mermaid
sequenceDiagram
    participant App
    participant URLRequest
    participant HSTS
    participant Transaction

    App->>URLRequest: new(http://google.com)
    URLRequest->>HSTS: should_upgrade("google.com")?
    HSTS-->>URLRequest: Yes (preloaded)
    URLRequest->>URLRequest: Upgrade to https://google.com
    URLRequest->>Transaction: start()
    Note over Transaction: Proceeds with HTTPS
```

---

## Scenario 3: Certificate Pinning

```mermaid
sequenceDiagram
    participant ConnectJob
    participant PinStore
    participant Server

    ConnectJob->>Server: TLS handshake
    Server-->>ConnectJob: Certificate chain
    ConnectJob->>ConnectJob: Compute SPKI hashes
    ConnectJob->>PinStore: check(host, hashes)
    alt Pins match
        PinStore-->>ConnectJob: Ok
        ConnectJob-->>App: Connection established
    else Pins mismatch
        PinStore-->>ConnectJob: Err(CertPinningFailed)
        ConnectJob-->>App: Error
    end
```

---

## Scenario 4: H2 Multiplexing

```mermaid
sequenceDiagram
    participant Req1
    participant Req2
    participant Factory
    participant Cache
    participant Server

    Req1->>Factory: create_stream(example.com)
    Factory->>Cache: lookup(example.com)?
    Cache-->>Factory: None
    Factory->>Server: New H2 connection
    Server-->>Factory: H2 sender
    Factory->>Cache: store(example.com, sender.clone())
    Factory-->>Req1: Stream

    Req2->>Factory: create_stream(example.com)
    Factory->>Cache: lookup(example.com)?
    Cache-->>Factory: Cached sender
    Factory-->>Req2: Multiplexed stream (no new connection!)
```

---

## Scenario 5: Cookie with PSL Validation

```mermaid
sequenceDiagram
    participant Server
    participant Transaction
    participant CookieMonster
    participant PSL

    Server-->>Transaction: Set-Cookie: session=abc; Domain=.com
    Transaction->>CookieMonster: parse_and_save_cookie()
    CookieMonster->>PSL: is_public_suffix(".com")?
    PSL-->>CookieMonster: Yes (public suffix!)
    CookieMonster-->>Transaction: Cookie REJECTED (supercookie attack)
```

---

## File Map

| Module | Files | Responsibility |
|--------|-------|----------------|
| `urlrequest` | request.rs, job.rs, context.rs, device.rs | Public API |
| `http` | transaction.rs, streamfactory.rs, retry.rs, h2settings.rs, orderedheaders.rs | HTTP/1.1 & H2 |
| `socket` | pool.rs, connectjob.rs, stream.rs, tls.rs, proxy.rs, authcache.rs | Connections |
| `cookies` | monster.rs, canonical_cookie.rs, persistence.rs, psl.rs, browser.rs, oscrypt.rs | Cookie state |
| `tls` | hsts.rs, pinning.rs, ct.rs | Security |
| `base` | neterror.rs, loadstate.rs | Common types |

---

## Test Coverage

| Module | Unit Tests | Integration Tests |
|--------|------------|-------------------|
| cookies | 17 | 6 (psl_test) |
| http | 20 | - |
| socket | 7 | 6 (authcache_test) |
| tls | 21 | 12 (hsts_test, pinning_test) |
| urlrequest | 9 | - |
| **Total** | **88** | **24** |
