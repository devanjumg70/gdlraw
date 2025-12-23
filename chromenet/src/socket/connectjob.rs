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
    pub async fn connect(url: &Url) -> Result<SocketType, NetError> {
        let host = url.host_str().ok_or(NetError::InvalidUrl)?;
        let port = url.port_or_known_default().ok_or(NetError::InvalidUrl)?;

        // 1. DNS Resolution (System for now, custom later)
        let addr_str = format!("{}:{}", host, port);
        // Using tokio's resolve
        let addrs =
            tokio::net::lookup_host(&addr_str).await.map_err(|_| NetError::NameNotResolved)?;

        // 2. TCP Connect (Happy Eyeballs - simple version)
        let mut stream = None;
        for addr in addrs {
            if let Ok(s) = TcpStream::connect(addr).await {
                stream = Some(s);
                break;
            }
        }

        let stream = stream.ok_or(NetError::ConnectionFailed)?;

        // 3. SSL Handshake (if https)
        if url.scheme() == "https" {
            let mut builder =
                SslConnector::builder(SslMethod::tls()).map_err(|_| NetError::SslProtocolError)?;

            // Apply Chromium settings
            use crate::socket::tls::TlsConfig;
            let tls_config = TlsConfig::default_chrome();
            tls_config.apply_to_builder(&mut builder)?;

            let connector = builder.build();
            let config = connector.configure().map_err(|_| NetError::SslProtocolError)?;

            let tls_stream = tokio_boring::connect(config, host, stream).await.map_err(|e| {
                eprintln!("SSL Handshake failed: {:?}", e);
                NetError::SslProtocolError
            })?;

            Ok(SocketType::Ssl(tls_stream))
        } else {
            Ok(SocketType::Tcp(stream))
        }
    }
}
