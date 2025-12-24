# Chromenet Codemap

Complete flow diagrams and scenarios for the chromenet library.

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
        RETRY[Retry Logic]
    end

    subgraph "Connection Layer"
        POOL[ClientSocketPool]
        CJ[ConnectJob]
        STREAM[StreamSocket]
    end

    subgraph "Transport Layer"
        TCP[TcpStream]
        TLS[SslStream]
        PROXY[Proxy Handshake]
    end

    subgraph "State Management"
        COOKIE[CookieMonster]
        PERSIST[Persistence]
    end

    URL --> CTX
    CTX --> TXN
    TXN --> RETRY
    TXN --> SF
    SF --> POOL
    POOL --> CJ
    CJ --> TCP
    CJ --> PROXY
    PROXY --> TLS
    TCP --> STREAM
    TLS --> STREAM

    TXN --> COOKIE
    COOKIE --> PERSIST
```

---

## Scenario 1: Simple HTTPS Request

```mermaid
sequenceDiagram
    participant App
    participant URLRequest
    participant Transaction
    participant Pool
    participant ConnectJob
    participant Server

    App->>URLRequest: new(https://example.com)
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
    Transaction->>CookieMonster: save cookies
    Transaction-->>App: Response
```

---

## Scenario 2: Request with HTTP Proxy

```mermaid
sequenceDiagram
    participant App
    participant Transaction
    participant ConnectJob
    participant Proxy
    participant Server

    App->>Transaction: start() with proxy
    Transaction->>ConnectJob: connect(proxy)
    ConnectJob->>Proxy: TCP connect
    ConnectJob->>Proxy: HTTP CONNECT example.com:443
    Proxy-->>ConnectJob: 200 Connection Established
    ConnectJob->>Server: TLS handshake (through tunnel)
    ConnectJob-->>Transaction: SslStream
    Transaction->>Server: HTTP request
    Server-->>Transaction: Response
```

---

## Scenario 3: Connection Retry with Backoff

```mermaid
sequenceDiagram
    participant App
    participant Transaction
    participant RetryLogic
    participant ConnectJob
    participant Server

    App->>Transaction: start()
    Transaction->>ConnectJob: connect()
    ConnectJob->>Server: TCP connect
    Server-->>ConnectJob: RST (connection reset)
    ConnectJob-->>Transaction: ConnectionReset

    Transaction->>RetryLogic: should_retry?
    RetryLogic-->>Transaction: Yes (attempt 1 < 3)
    Transaction->>Transaction: sleep(100ms)

    Transaction->>ConnectJob: connect() retry
    ConnectJob->>Server: TCP connect
    Server-->>ConnectJob: Success
    Transaction-->>App: Response
```

---

## Scenario 4: Socket Pool Reuse

```mermaid
sequenceDiagram
    participant Req1
    participant Req2
    participant Pool
    participant Server

    Req1->>Pool: request_socket(example.com)
    Pool->>Server: New connection
    Server-->>Pool: SslStream
    Pool-->>Req1: Stream

    Req1->>Req1: Complete request
    Req1->>Pool: release_socket(stream)

    Req2->>Pool: request_socket(example.com)
    Pool-->>Req2: Reused stream (no connect)
```

---

## Scenario 5: Cookie Flow

```mermaid
sequenceDiagram
    participant Transaction
    participant CookieMonster
    participant Disk

    Transaction->>CookieMonster: get_cookies_for_url()
    CookieMonster-->>Transaction: [session=abc]
    Transaction->>Server: Request + Cookie header

    Server-->>Transaction: Response + Set-Cookie: token=xyz

    Transaction->>CookieMonster: parse_and_save_cookie()
    CookieMonster->>CookieMonster: enforce_per_domain_limit(50)
    CookieMonster->>CookieMonster: enforce_global_limit(3000)

    Note over CookieMonster,Disk: Optional persistence
    CookieMonster->>Disk: save_cookies()
```

---

## File Map

| Module | Files | Responsibility |
|--------|-------|----------------|
| `urlrequest` | request.rs, job.rs, context.rs, device.rs | Public API |
| `http` | transaction.rs, streamfactory.rs, retry.rs | HTTP/1.1 & H2 |
| `socket` | pool.rs, connectjob.rs, stream.rs, tls.rs, proxy.rs | Connections |
| `cookies` | monster.rs, canonical_cookie.rs, persistence.rs | Cookie state |
| `base` | neterror.rs, loadstate.rs | Common types |
