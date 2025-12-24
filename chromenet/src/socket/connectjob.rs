use crate::base::neterror::NetError;
use crate::socket::stream::{BoxedSocket, StreamSocket};
use crate::socket::tls::TlsConfig;
use boring::ssl::{SslConnector, SslMethod};
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_boring::SslStream;
use url::Url;

/// Chromium's Happy Eyeballs IPv6 fallback delay (250ms).
const IPV6_FALLBACK_DELAY: Duration = Duration::from_millis(250);

/// Connection timeout (4 minutes, matches Chromium).
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(240);

/// Result of a connection attempt, includes ALPN negotiation info.
pub struct ConnectResult {
    pub socket: BoxedSocket,
    /// True if HTTP/2 was negotiated via ALPN.
    pub is_h2: bool,
}

/// Manages the connection process: DNS -> TCP -> SSL.
/// Implements Happy Eyeballs (RFC 8305) for faster dual-stack connections.
/// Supports HTTPS proxies with TLS-in-TLS tunneling.
pub struct ConnectJob;

impl ConnectJob {
    /// Connect to the target URL, optionally through a proxy.
    /// Returns a BoxedSocket for polymorphic handling (supports TLS-in-TLS).
    pub async fn connect(
        url: &Url,
        proxy: Option<&crate::socket::proxy::ProxySettings>,
    ) -> Result<ConnectResult, NetError> {
        match proxy {
            Some(p) => match p.proxy_type() {
                crate::socket::proxy::ProxyType::Http => Self::http_proxy_connect(url, p).await,
                crate::socket::proxy::ProxyType::Https => Self::https_proxy_connect(url, p).await,
                crate::socket::proxy::ProxyType::Socks5 => Self::socks5_proxy_connect(url, p).await,
            },
            None => Self::direct_connect(url).await,
        }
    }

    /// Direct connection (no proxy).
    async fn direct_connect(url: &Url) -> Result<ConnectResult, NetError> {
        let host = url.host_str().ok_or(NetError::InvalidUrl)?;
        let port = url.port_or_known_default().ok_or(NetError::InvalidUrl)?;

        // TCP connect with Happy Eyeballs
        let tcp = Self::connect_tcp(host, port).await?;

        // TLS if HTTPS
        if url.scheme() == "https" {
            let (tls, is_h2) = Self::ssl_handshake(tcp, host).await?;
            Ok(ConnectResult { socket: BoxedSocket::new(tls), is_h2 })
        } else {
            Ok(ConnectResult { socket: BoxedSocket::new(tcp), is_h2: false })
        }
    }

    /// HTTP proxy connection (plain TCP to proxy, CONNECT, then TLS to target).
    async fn http_proxy_connect(
        url: &Url,
        proxy: &crate::socket::proxy::ProxySettings,
    ) -> Result<ConnectResult, NetError> {
        let proxy_host = proxy.url.host_str().ok_or(NetError::InvalidUrl)?;
        let proxy_port = proxy.url.port_or_known_default().ok_or(NetError::InvalidUrl)?;

        // Step 1: TCP to proxy
        let mut tcp = Self::connect_tcp(proxy_host, proxy_port).await?;

        // Step 2: HTTP CONNECT tunnel
        Self::send_connect(&mut tcp, url, proxy).await?;

        // Step 3: TLS to target if HTTPS
        if url.scheme() == "https" {
            let target_host = url.host_str().ok_or(NetError::InvalidUrl)?;
            let (tls, is_h2) = Self::ssl_handshake(tcp, target_host).await?;
            Ok(ConnectResult { socket: BoxedSocket::new(tls), is_h2 })
        } else {
            Ok(ConnectResult { socket: BoxedSocket::new(tcp), is_h2: false })
        }
    }

    /// HTTPS proxy connection (TLS-in-TLS tunneling).
    /// Flow: TCP -> TLS(proxy) -> CONNECT -> TLS(target)
    async fn https_proxy_connect(
        url: &Url,
        proxy: &crate::socket::proxy::ProxySettings,
    ) -> Result<ConnectResult, NetError> {
        let proxy_host = proxy.url.host_str().ok_or(NetError::InvalidUrl)?;
        let proxy_port = proxy.url.port_or_known_default().ok_or(NetError::InvalidUrl)?;

        // Step 1: TCP to proxy
        let tcp = Self::connect_tcp(proxy_host, proxy_port).await?;

        // Step 2: TLS to proxy (Layer 1)
        let (mut proxy_tls, _) = Self::ssl_handshake(tcp, proxy_host).await?;

        // Step 3: HTTP CONNECT through TLS tunnel
        Self::send_connect_generic(&mut proxy_tls, url, proxy).await?;

        // Step 4: TLS to target through tunnel (Layer 2 - TLS-in-TLS)
        if url.scheme() == "https" {
            let target_host = url.host_str().ok_or(NetError::InvalidUrl)?;
            let (target_tls, is_h2) = Self::ssl_handshake_generic(proxy_tls, target_host).await?;
            Ok(ConnectResult { socket: BoxedSocket::new(target_tls), is_h2 })
        } else {
            Ok(ConnectResult { socket: BoxedSocket::new(proxy_tls), is_h2: false })
        }
    }

    /// SOCKS5 proxy connection.
    async fn socks5_proxy_connect(
        url: &Url,
        proxy: &crate::socket::proxy::ProxySettings,
    ) -> Result<ConnectResult, NetError> {
        let proxy_host = proxy.url.host_str().ok_or(NetError::InvalidUrl)?;
        let proxy_port = proxy.url.port_or_known_default().ok_or(NetError::InvalidUrl)?;

        // Step 1: TCP to proxy
        let mut tcp = Self::connect_tcp(proxy_host, proxy_port).await?;

        // Step 2: SOCKS5 handshake
        Self::socks5_handshake(&mut tcp, url).await?;

        // Step 3: TLS to target if HTTPS
        if url.scheme() == "https" {
            let target_host = url.host_str().ok_or(NetError::InvalidUrl)?;
            let (tls, is_h2) = Self::ssl_handshake(tcp, target_host).await?;
            Ok(ConnectResult { socket: BoxedSocket::new(tls), is_h2 })
        } else {
            Ok(ConnectResult { socket: BoxedSocket::new(tcp), is_h2: false })
        }
    }

    /// TCP connect with Happy Eyeballs (RFC 8305).
    async fn connect_tcp(host: &str, port: u16) -> Result<TcpStream, NetError> {
        let addr_str = format!("{}:{}", host, port);
        let addrs: Vec<SocketAddr> = tokio::net::lookup_host(&addr_str)
            .await
            .map_err(|_| NetError::NameNotResolved)?
            .collect();

        if addrs.is_empty() {
            return Err(NetError::NameNotResolved);
        }

        Self::connect_with_happy_eyeballs(&addrs).await
    }

    /// Connect using Happy Eyeballs (RFC 8305).
    async fn connect_with_happy_eyeballs(addrs: &[SocketAddr]) -> Result<TcpStream, NetError> {
        let (ipv6_addrs, ipv4_addrs): (Vec<_>, Vec<_>) =
            addrs.iter().partition(|a| matches!(a.ip(), IpAddr::V6(_)));

        if ipv6_addrs.is_empty() {
            return Self::connect_any(&ipv4_addrs).await;
        }
        if ipv4_addrs.is_empty() {
            return Self::connect_any(&ipv6_addrs).await;
        }

        tokio::select! {
            result = Self::connect_any(&ipv6_addrs) => {
                match result {
                    Ok(stream) => Ok(stream),
                    Err(_) => Self::connect_any(&ipv4_addrs).await,
                }
            }
            result = async {
                tokio::time::sleep(IPV6_FALLBACK_DELAY).await;
                Self::connect_any(&ipv4_addrs).await
            } => {
                result
            }
        }
    }

    async fn connect_any(addrs: &[&SocketAddr]) -> Result<TcpStream, NetError> {
        let mut last_error = NetError::ConnectionFailed;
        for addr in addrs {
            match tokio::time::timeout(CONNECTION_TIMEOUT, TcpStream::connect(addr)).await {
                Ok(Ok(stream)) => return Ok(stream),
                Ok(Err(_)) => last_error = NetError::ConnectionRefused,
                Err(_) => last_error = NetError::ConnectionTimedOut,
            }
        }
        Err(last_error)
    }

    /// SSL handshake for TcpStream, returns (SslStream, is_h2).
    async fn ssl_handshake(
        stream: TcpStream,
        host: &str,
    ) -> Result<(SslStream<TcpStream>, bool), NetError> {
        let mut builder =
            SslConnector::builder(SslMethod::tls()).map_err(|_| NetError::SslProtocolError)?;

        // ALPN for H2 and H1
        let protos = b"\x02h2\x08http/1.1";
        builder.set_alpn_protos(protos).map_err(|_| NetError::SslProtocolError)?;

        // Apply Chrome TLS settings
        let tls_config = TlsConfig::default_chrome();
        tls_config.apply_to_builder(&mut builder)?;

        let connector = builder.build();
        let config = connector.configure().map_err(|_| NetError::SslProtocolError)?;

        let tls_stream = tokio_boring::connect(config, host, stream).await.map_err(|e| {
            eprintln!("SSL Handshake failed: {:?}", e);
            NetError::SslProtocolError
        })?;

        let is_h2 = matches!(tls_stream.ssl().selected_alpn_protocol(), Some(b"h2"));
        Ok((tls_stream, is_h2))
    }

    /// Generic SSL handshake for any StreamSocket (enables TLS-in-TLS).
    async fn ssl_handshake_generic<S: StreamSocket>(
        stream: S,
        host: &str,
    ) -> Result<(SslStream<S>, bool), NetError> {
        let mut builder =
            SslConnector::builder(SslMethod::tls()).map_err(|_| NetError::SslProtocolError)?;

        let protos = b"\x02h2\x08http/1.1";
        builder.set_alpn_protos(protos).map_err(|_| NetError::SslProtocolError)?;

        let tls_config = TlsConfig::default_chrome();
        tls_config.apply_to_builder(&mut builder)?;

        let connector = builder.build();
        let config = connector.configure().map_err(|_| NetError::SslProtocolError)?;

        let tls_stream = tokio_boring::connect(config, host, stream).await.map_err(|_| {
            eprintln!("SSL Handshake (TLS-in-TLS) failed for host: {}", host);
            NetError::SslProtocolError
        })?;

        let is_h2 = matches!(tls_stream.ssl().selected_alpn_protocol(), Some(b"h2"));
        Ok((tls_stream, is_h2))
    }

    /// Send HTTP CONNECT through a TcpStream.
    async fn send_connect(
        stream: &mut TcpStream,
        url: &Url,
        proxy: &crate::socket::proxy::ProxySettings,
    ) -> Result<(), NetError> {
        Self::send_connect_impl(stream, url, proxy).await
    }

    /// Send HTTP CONNECT through any StreamSocket (for TLS tunnels).
    async fn send_connect_generic<S: StreamSocket>(
        stream: &mut S,
        url: &Url,
        proxy: &crate::socket::proxy::ProxySettings,
    ) -> Result<(), NetError> {
        Self::send_connect_impl(stream, url, proxy).await
    }

    /// HTTP CONNECT implementation (generic over any AsyncRead+AsyncWrite).
    async fn send_connect_impl<S>(
        stream: &mut S,
        url: &Url,
        proxy: &crate::socket::proxy::ProxySettings,
    ) -> Result<(), NetError>
    where
        S: AsyncReadExt + AsyncWriteExt + Unpin,
    {
        let target_host = url.host_str().ok_or(NetError::InvalidUrl)?;
        let target_port = url.port_or_known_default().ok_or(NetError::InvalidUrl)?;
        let target = format!("{}:{}", target_host, target_port);

        let mut connect_req = format!("CONNECT {} HTTP/1.1\r\nHost: {}\r\n", target, target);

        if let Some(auth) = proxy.get_auth_header() {
            connect_req.push_str(&format!("Proxy-Authorization: {}\r\n", auth));
        }
        connect_req.push_str("\r\n");

        stream.write_all(connect_req.as_bytes()).await.map_err(|_| NetError::ConnectionFailed)?;

        // Read response
        let mut response = Vec::with_capacity(1024);
        let mut buf = [0u8; 256];

        loop {
            let n = stream.read(&mut buf).await.map_err(|_| NetError::ConnectionFailed)?;
            if n == 0 {
                return Err(NetError::EmptyResponse);
            }
            response.extend_from_slice(&buf[..n]);

            if response.windows(4).any(|w| w == b"\r\n\r\n") {
                break;
            }
            if response.len() > 8192 {
                return Err(NetError::ResponseHeadersTooBig);
            }
        }

        let response_str = String::from_utf8_lossy(&response);
        if !response_str.starts_with("HTTP/1.1 200") && !response_str.starts_with("HTTP/1.0 200") {
            eprintln!("Proxy Tunnel Failed: {}", response_str);
            return Err(NetError::TunnelConnectionFailed);
        }

        Ok(())
    }

    /// SOCKS5 handshake (RFC 1928).
    async fn socks5_handshake(stream: &mut TcpStream, url: &Url) -> Result<(), NetError> {
        const SOCKS5_VERSION: u8 = 0x05;
        const NO_AUTH: u8 = 0x00;
        const CONNECT_CMD: u8 = 0x01;
        const DOMAIN_ADDR: u8 = 0x03;

        let target_host = url.host_str().ok_or(NetError::InvalidUrl)?;
        let target_port = url.port_or_known_default().ok_or(NetError::InvalidUrl)?;

        if target_host.len() > 255 {
            return Err(NetError::InvalidUrl);
        }

        // Phase 1: Greeting
        let greet = [SOCKS5_VERSION, 0x01, NO_AUTH];
        stream.write_all(&greet).await.map_err(|_| NetError::ConnectionFailed)?;

        let mut greet_response = [0u8; 2];
        stream
            .read_exact(&mut greet_response)
            .await
            .map_err(|_| NetError::SocksConnectionFailed)?;

        if greet_response[0] != SOCKS5_VERSION || greet_response[1] != NO_AUTH {
            return Err(NetError::SocksConnectionFailed);
        }

        // Phase 2: Connect request
        let mut handshake = Vec::with_capacity(7 + target_host.len());
        handshake.push(SOCKS5_VERSION);
        handshake.push(CONNECT_CMD);
        handshake.push(0x00);
        handshake.push(DOMAIN_ADDR);
        handshake.push(target_host.len() as u8);
        handshake.extend_from_slice(target_host.as_bytes());
        handshake.push((target_port >> 8) as u8);
        handshake.push((target_port & 0xFF) as u8);

        stream.write_all(&handshake).await.map_err(|_| NetError::ConnectionFailed)?;

        // Read response
        let mut response_header = [0u8; 5];
        stream
            .read_exact(&mut response_header)
            .await
            .map_err(|_| NetError::SocksConnectionFailed)?;

        if response_header[0] != SOCKS5_VERSION || response_header[1] != 0x00 {
            return Err(NetError::SocksConnectionFailed);
        }

        // Drain remaining address bytes
        let addr_type = response_header[3];
        let remaining_bytes = match addr_type {
            0x01 => 4 + 2 - 1,
            0x03 => response_header[4] as usize + 2,
            0x04 => 16 + 2 - 1,
            _ => return Err(NetError::SocksConnectionFailed),
        };

        let mut remaining = vec![0u8; remaining_bytes];
        stream.read_exact(&mut remaining).await.map_err(|_| NetError::SocksConnectionFailed)?;

        Ok(())
    }
}
