# QUIC / HTTP/3 Module

Type structure for QUIC transport and HTTP/3 protocol.

> **Note**: Full integration requires the `quinn` crate (commented in Cargo.toml as optional).

## Configuration

```rust
use chromenet::quic::QuicConfig;

let config = QuicConfig::new()
    .idle_timeout(Duration::from_secs(30))
    .enable_0rtt(true)
    .initial_max_data(10 * 1024 * 1024)
    .alpn_protocols(vec!["h3".to_string()]);
```

### QuicConfig Options

| Option | Default | Description |
|--------|---------|-------------|
| `idle_timeout` | 60s | Max connection idle time |
| `initial_rtt` | 100ms | Initial RTT estimate |
| `max_udp_payload_size` | 1200 | Max UDP payload |
| `initial_max_data` | 10MB | Connection-level flow control |
| `initial_max_stream_data` | 1MB | Stream-level flow control |
| `initial_max_streams_bidi` | 100 | Max bidirectional streams |
| `initial_max_streams_uni` | 100 | Max unidirectional streams |
| `enable_0rtt` | true | Enable 0-RTT resumption |
| `alpn_protocols` | ["h3"] | ALPN protocols |

## Connection Builder

```rust
use chromenet::quic::QuicConnectionBuilder;

let conn = QuicConnectionBuilder::new()
    .url("https://example.com")?
    .config(config)
    .connect()
    .await?;
```

## Integration

To enable full QUIC support, uncomment in Cargo.toml:

```toml
quinn = { version = "0.11", optional = true }
```

The module is designed for seamless quinn integration when enabled.
