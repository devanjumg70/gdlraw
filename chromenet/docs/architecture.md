# chromenet Architecture

## Overview

chromenet is a Chromium-inspired HTTP networking library that provides browser-grade networking with full fingerprint control.

## Layer Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                     Public API Layer                         │
│  Client, RequestBuilder, URLRequest                         │
├─────────────────────────────────────────────────────────────┤
│                     HTTP Layer                               │
│  HttpNetworkTransaction, HttpCache, Multipart, Streaming    │
├─────────────────────────────────────────────────────────────┤
│                     Protocol Layer                           │
│  HttpStreamFactory (H1/H2), WebSocket, QUIC                 │
├─────────────────────────────────────────────────────────────┤
│                     Connection Layer                         │
│  ClientSocketPool, ConnectJob, Proxy                        │
├─────────────────────────────────────────────────────────────┤
│                     Security Layer                           │
│  BoringSSL, HSTS, Pinning, CT                               │
├─────────────────────────────────────────────────────────────┤
│                     State Layer                              │
│  CookieMonster, AuthCache, DNS                              │
└─────────────────────────────────────────────────────────────┘
```

## Module Map

| Module | Purpose | Key Types |
|--------|---------|-----------|
| `base` | Core types | `NetError`, `LoadState` |
| `client` | High-level API | `Client`, `ClientBuilder` |
| `cookies` | Cookie storage | `CookieMonster`, `CanonicalCookie` |
| `dns` | Resolution | `HickoryResolver` |
| `emulation` | Browser profiles | `Emulation`, `Device` |
| `http` | HTTP handling | `HttpNetworkTransaction`, `HttpCache` |
| `socket` | Connections | `ClientSocketPool`, `ConnectJob` |
| `tls` | Security | `HstsStore`, `PinSet` |
| `ws` | WebSocket | `WebSocket`, `Message` |
| `quic` | HTTP/3 | `QuicConfig`, `QuicConnection` |

## Request Flow

```
1. Client::get("url")
   └── RequestBuilder created

2. RequestBuilder::send()
   └── URLRequest::new()
       └── URLRequestHttpJob::start()

3. HttpNetworkTransaction::start()
   ├── Check HttpCache (if cached, return)
   ├── HttpStreamFactory::create_stream()
   │   ├── Check H2 session cache
   │   └── ClientSocketPool::request_socket()
   │       └── ConnectJob (DNS → TCP → TLS)
   └── stream.send_request(body)

4. Response processing
   ├── Handle Set-Cookie headers
   ├── Handle HSTS headers
   ├── Handle redirects
   └── Return to caller
```

## Key Design Decisions

1. **BoringSSL Only**: Matches Chromium's TLS fingerprint exactly
2. **DashMap**: Thread-safe concurrent access for pools and caches
3. **Builder Pattern**: Ergonomic configuration (Client, Emulation, Request)
4. **State Machines**: Transaction uses enum-based state machine like Chromium
5. **Strict Chromium Mapping**: Module names mirror `//net` structure
