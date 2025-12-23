use crate::base::neterror::NetError;
use crate::socket::client::SocketType;
use boring::ssl::{SslConnector, SslMethod};
use tokio::net::TcpStream;
use url::Url;

/// Manages the connection process: DNS -> TCP -> SSL.
/// Roughly equivalent to net::ConnectJob.
pub struct ConnectJob {
    // Configuration
}

impl ConnectJob {
    pub async fn connect(
        url: &Url,
        proxy: Option<&crate::socket::proxy::ProxySettings>,
    ) -> Result<SocketType, NetError> {
        let (host, port) = if let Some(p) = proxy {
            // If proxy, we connect to PROXY host/port first
            let phost = p.url.host_str().ok_or(NetError::InvalidUrl)?;
            let pport = p.url.port_or_known_default().ok_or(NetError::InvalidUrl)?;
            (phost, pport)
        } else {
            // Direct connection
            let dhost = url.host_str().ok_or(NetError::InvalidUrl)?;
            let dport = url.port_or_known_default().ok_or(NetError::InvalidUrl)?;
            (dhost, dport)
        };

        // 1. DNS Resolution
        let addr_str = format!("{}:{}", host, port);
        // Using tokio's resolve
        let addrs =
            tokio::net::lookup_host(&addr_str).await.map_err(|_| NetError::NameNotResolved)?;

        // 2. TCP Connect (to proxy or destination)
        let mut stream = None;
        for addr in addrs {
            if let Ok(s) = TcpStream::connect(addr).await {
                stream = Some(s);
                break;
            }
        }

        let mut stream = stream.ok_or(NetError::ConnectionFailed)?;

        // 2b. Proxy Handshake (HTTP CONNECT)
        // If we are using a proxy, we need to tunnel to the final destination.
        // We act as if we are connecting to the final host.
        if let Some(p) = proxy {
            match p.proxy_type() {
                crate::socket::proxy::ProxyType::Http => {
                    use tokio::io::{AsyncReadExt, AsyncWriteExt};

                    let target_host = url.host_str().ok_or(NetError::InvalidUrl)?;
                    let target_port = url.port_or_known_default().ok_or(NetError::InvalidUrl)?;
                    let target = format!("{}:{}", target_host, target_port);

                    let mut connect_req =
                        format!("CONNECT {} HTTP/1.1\r\nHost: {}\r\n", target, target);

                    if let Some(auth) = p.get_auth_header() {
                        connect_req.push_str(&format!("Proxy-Authorization: {}\r\n", auth));
                    }
                    connect_req.push_str("\r\n");

                    stream
                        .write_all(connect_req.as_bytes())
                        .await
                        .map_err(|_| NetError::ConnectionFailed)?;

                    // Read response
                    // Naive reading of headers
                    let mut buf = [0u8; 1024];
                    let n = stream.read(&mut buf).await.map_err(|_| NetError::ConnectionFailed)?;
                    let response = String::from_utf8_lossy(&buf[..n]);

                    if !response.starts_with("HTTP/1.1 200")
                        && !response.starts_with("HTTP/1.0 200")
                    {
                        eprintln!("Proxy Tunnel Failed: {}", response);
                        return Err(NetError::ConnectionRefused); // Or ProxyError
                    }

                    // Tunnel established! `stream` is now a tunnel to target.
                }
                _ => {
                    // TODO: Implement SOCKS5 or HTTPS proxy logic
                    eprintln!("Unsupported proxy type");
                    return Err(NetError::ConnectionFailed);
                }
            }
        }

        let target_host = url.host_str().ok_or(NetError::InvalidUrl)?;

        // 3. SSL Handshake (if https) - always happens *after* any tunnel is established
        if url.scheme() == "https" {
            let mut builder =
                SslConnector::builder(SslMethod::tls()).map_err(|_| NetError::SslProtocolError)?;

            // Configure ALPN (H2 and H1)
            // "\x02h2\x08http/1.1"
            let protos = b"\x02h2\x08http/1.1";
            builder.set_alpn_protos(protos).map_err(|_| NetError::SslProtocolError)?;

            // Apply Chromium settings
            use crate::socket::tls::TlsConfig;
            let tls_config = TlsConfig::default_chrome();
            tls_config.apply_to_builder(&mut builder)?;

            let connector = builder.build();
            let config = connector.configure().map_err(|_| NetError::SslProtocolError)?;

            let tls_stream =
                tokio_boring::connect(config, target_host, stream).await.map_err(|e| {
                    eprintln!("SSL Handshake failed: {:?}", e);
                    NetError::SslProtocolError
                })?;

            Ok(SocketType::Ssl(tls_stream))
        } else {
            Ok(SocketType::Tcp(stream))
        }
    }
}
