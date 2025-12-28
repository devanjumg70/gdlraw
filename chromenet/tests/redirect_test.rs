use chromenet::urlrequest::request::URLRequest;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::test]
async fn test_redirect_limit() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let base_url = format!("http://{}", addr);
    let server_url = base_url.clone();

    tokio::spawn(async move {
        loop {
            if let Ok((mut socket, _)) = listener.accept().await {
                let server_url = server_url.clone();
                tokio::spawn(async move {
                    let mut buf = [0u8; 1024];
                    let _ = socket.read(&mut buf).await;
                    let response = format!(
                        "HTTP/1.1 302 Found\r\nLocation: {}/loop\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                        server_url
                    );
                    let _ = socket.write_all(response.as_bytes()).await;
                });
            }
        }
    });

    let mut req = URLRequest::new(&format!("{}/start", base_url)).unwrap();
    let result = req.start().await;
    assert!(result.is_err(), "Should fail with TooManyRedirects");
}

#[tokio::test]
async fn test_redirect_persists_proxy() {
    // 1. Setup Redirect Server
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let base_url = format!("http://{}", addr);
    let server_url = base_url.clone();
    let server_port = addr.port();

    tokio::spawn(async move {
        loop {
            if let Ok((mut socket, _)) = listener.accept().await {
                let server_url = server_url.clone();
                tokio::spawn(async move {
                    let mut buf = [0u8; 4096];
                    let n = socket.read(&mut buf).await.unwrap_or(0);
                    let request = String::from_utf8_lossy(&buf[..n]);

                    if request.contains("GET /start") {
                        // Return 302 and CLOSE connection to force new Proxy Connection
                        let response = format!(
                            "HTTP/1.1 302 Found\r\nLocation: {}/target\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                            server_url
                        );
                        let _ = socket.write_all(response.as_bytes()).await;
                    } else {
                        let response = "HTTP/1.1 200 OK\r\nContent-Length: 6\r\nConnection: close\r\n\r\nTARGET";
                        let _ = socket.write_all(response.as_bytes()).await;
                    }
                });
            }
        }
    });

    // 2. Setup Proxy Listener
    let proxy_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let proxy_addr = proxy_listener.local_addr().unwrap();
    let proxy_hit_count = Arc::new(AtomicUsize::new(0));
    let proxy_hit_count_clone = proxy_hit_count.clone();

    tokio::spawn(async move {
        loop {
            if let Ok((mut client_sock, _)) = proxy_listener.accept().await {
                let proxy_hit_count = proxy_hit_count_clone.clone();
                proxy_hit_count.fetch_add(1, Ordering::Relaxed);

                tokio::spawn(async move {
                    if let Ok(mut server_sock) =
                        tokio::net::TcpStream::connect(format!("127.0.0.1:{}", server_port)).await
                    {
                        let (mut cr, mut cw) = client_sock.split();
                        let (mut sr, mut sw) = server_sock.split();
                        let _ = tokio::join!(
                            tokio::io::copy(&mut cr, &mut sw),
                            tokio::io::copy(&mut sr, &mut cw)
                        );
                    }
                });
            }
        }
    });

    // 3. Request
    let mut req = URLRequest::new(&format!("{}/start", base_url)).unwrap();
    let proxy = chromenet::socket::proxy::ProxyBuilder::new()
        .url(&format!("http://{}", proxy_addr))
        .build()
        .unwrap();
    req.set_proxy(proxy);

    let _ = req.start().await;

    // 4. Assert
    let count = proxy_hit_count.load(Ordering::Relaxed);
    assert_eq!(
        count, 2,
        "Proxy should be used 2 times (Start + Redirect). Got {}",
        count
    );
}

#[tokio::test]
async fn test_redirect_strips_auth_cross_origin() {
    let listener_a = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr_a = listener_a.local_addr().unwrap();

    let listener_b = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr_b = listener_b.local_addr().unwrap();
    let url_b = format!("http://{}", addr_b);

    tokio::spawn(async move {
        loop {
            if let Ok((mut socket, _)) = listener_a.accept().await {
                let url_b = url_b.clone();
                tokio::spawn(async move {
                    let mut buf = [0u8; 4096];
                    let _ = socket.read(&mut buf).await;
                    let response = format!(
                        "HTTP/1.1 302 Found\r\nLocation: {}/target\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                        url_b
                     );
                    let _ = socket.write_all(response.as_bytes()).await;
                });
            }
        }
    });

    tokio::spawn(async move {
        loop {
            if let Ok((mut socket, _)) = listener_b.accept().await {
                tokio::spawn(async move {
                    let mut buf = [0u8; 4096];
                    let n = socket.read(&mut buf).await.unwrap_or(0);
                    let request = String::from_utf8_lossy(&buf[..n]);
                    let request_lower = request.to_lowercase();

                    if request_lower.contains("authorization: secret") {
                        let response = "HTTP/1.1 403 Forbidden\r\nContent-Length: 12\r\nConnection: close\r\n\r\nAuth Leaked!";
                        let _ = socket.write_all(response.as_bytes()).await;
                    } else {
                        let response =
                            "HTTP/1.1 200 OK\r\nContent-Length: 4\r\nConnection: close\r\n\r\nSafe";
                        let _ = socket.write_all(response.as_bytes()).await;
                    }
                });
            }
        }
    });

    let mut req = URLRequest::new(&format!("http://{}/start", addr_a)).unwrap();
    req.add_header("Authorization", "Secret");

    let _ = req.start().await;
    let resp = req.get_response().expect("Should succeed");

    assert_eq!(
        resp.status(),
        200,
        "Should be 200 OK (Safe). If 403, Auth leaked."
    );
}

#[tokio::test]
async fn test_redirect_persists_headers_same_origin() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let base_url = format!("http://{}", addr);
    let server_url = base_url.clone();

    tokio::spawn(async move {
        loop {
            if let Ok((mut socket, _)) = listener.accept().await {
                let server_url = server_url.clone();
                tokio::spawn(async move {
                    let mut buf = [0u8; 4096];
                    let n = socket.read(&mut buf).await.unwrap_or(0);
                    let request = String::from_utf8_lossy(&buf[..n]);
                    let request_lower = request.to_lowercase();

                    if request.contains("GET /start") {
                        let response = format!(
                            "HTTP/1.1 302 Found\r\nLocation: {}/target\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                            server_url
                         );
                        let _ = socket.write_all(response.as_bytes()).await;
                    } else if request.contains("GET /target") {
                        if request_lower.contains("x-custom: foo") {
                            let response = "HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nFound";
                            let _ = socket.write_all(response.as_bytes()).await;
                        } else {
                            eprintln!("Headers missing. Got:\n{}", request);
                            let response = "HTTP/1.1 400 Bad Request\r\nContent-Length: 14\r\nConnection: close\r\n\r\nMissing Header";
                            let _ = socket.write_all(response.as_bytes()).await;
                        }
                    }
                });
            }
        }
    });

    let mut req = URLRequest::new(&format!("{}/start", base_url)).unwrap();
    req.add_header("X-Custom", "Foo");
    let _ = req.start().await;
    let resp = req.get_response().expect("Should succeed");

    assert_eq!(
        resp.status(),
        200,
        "Custom header should persist on same-origin redirect"
    );
}
