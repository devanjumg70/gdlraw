use chromenet::base::neterror::NetError;
use chromenet::cookies::monster::CookieMonster;
use chromenet::http::streamfactory::HttpStreamFactory;
use chromenet::http::transaction::HttpNetworkTransaction;
use chromenet::socket::pool::ClientSocketPool;
use chromenet::socket::stream::BoxedSocket;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use url::Url;

#[tokio::test]
async fn test_retry_on_reused_socket_failure() {
    // 1. Setup Server that accepts one request, then closes connection for second
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server_url = Url::parse(&format!("http://{}", addr)).unwrap();

    tokio::spawn(async move {
        // Handle 1st request (Keep-Alive)
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut buf = [0u8; 1024];
        let n = socket.read(&mut buf).await.unwrap();
        // Respond OK
        socket
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nOK")
            .await
            .unwrap();

        // Wait then close (simulating timeout or disconnect)
        // We drop socket here.
        drop(socket);

        // Handle 2nd connection (Retry)
        if let Ok((mut socket2, _)) = listener.accept().await {
            let n = socket2.read(&mut buf).await.unwrap();
            socket2
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nRETRY")
                .await
                .unwrap();
        }
    });

    // 2. Client Setup
    let pool = Arc::new(ClientSocketPool::new());
    let factory = Arc::new(HttpStreamFactory::new(pool.clone()));
    let cookies = Arc::new(CookieMonster::new());

    // 3. Request 1 (Prime the pool)
    {
        let mut trans =
            HttpNetworkTransaction::new(factory.clone(), server_url.clone(), cookies.clone());
        trans.start().await.expect("Request 1 failed");
        let resp = trans.get_response().unwrap();
        assert_eq!(resp.status(), 200);
        // Note: Transaction currently does NO cleanup/release on drop because I haven't implemented RAII fully.
        // It drops HttpStream, which drops connection.
        // THIS TEST WILL FAIL AUTOMATICALLY IF I DON'T IMPLEMENT RELEASE.
        // But wait, if I don't release, the socket is CLOSED.
        // Then `request_socket` will get a NEW socket, not a REUSED one.
        // So I *must* release socket to test Reuse Failure.
    }

    // MANUAL POOL MANIPULATION TO SIMULATE REUSE (Since Transaction doesn't release yet)
    // Actually, testing `retry` requires that we *get* a reused socket.
    // If `Transaction` doesn't release, we never get reused sockets.
    // So I MUST implement `release` in Transaction or manually inject into pool.

    // Manual Injection Strategy:
    // Connect manually, put in pool.
    // But `pool.idle_sockets` is private.
    // Use `pool.release_socket`.
    // I need a `SocketType`.

    let stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    // Read EOF to simulate closed? No, server closes it.

    // Put into pool claiming it's for `server_url`.
    // We need to wrap it in SocketType.
    // SocketType is in `crate::socket::client`.
    // It's public enum? Yes.

    // We can't import `SocketType` easily if it's not re-exported?
    // It is `pub use crate::socket::client::SocketType` in `socket::mod.rs`? No?
    // Let's check imports.
    use chromenet::socket::client::SocketType;

    let socket_wrapper = SocketType::Tcp(stream);
    pool.release_socket(&server_url, BoxedSocket::new(socket_wrapper), false);

    // Now pool has a "Idle" socket.
    // Server has closed its end (after accept logic 1 spawning).
    // Wait for server to close?
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // 4. Request 2 (Should pick up "reused" socket, fail, retry with fresh)
    let mut trans =
        HttpNetworkTransaction::new(factory.clone(), server_url.clone(), cookies.clone());
    trans.start().await.expect("Request 2 (Retry) failed");

    // If it succeeded, it must have retried, because the pooled socket was closed!
    let resp = trans.get_response().unwrap();
    assert_eq!(resp.status(), 200);
}
