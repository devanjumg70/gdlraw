use chromenet::base::neterror::NetError;
use chromenet::socket::pool::ClientSocketPool;
use tokio::net::TcpListener;
use url::Url;

#[tokio::test]
async fn test_pool_limits() {
    // 1. Start a local server to connect to
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let url_str = format!("http://127.0.0.1:{}/", port);
    let url = Url::parse(&url_str).unwrap();

    let pool = ClientSocketPool::new();

    // Spawn a background task to accept connections so ConnectJob doesn't hang or fail
    tokio::spawn(async move {
        while (listener.accept().await).is_ok() {
            // Do nothing, just accept
        }
    });

    // 2. Consume all 6 slots
    let mut sockets = Vec::new();
    for _ in 0..6 {
        let socket_res = pool.request_socket(&url, None).await;
        assert!(socket_res.is_ok(), "Failed to acquire socket within limit");
        let result = socket_res.unwrap();
        sockets.push(result.socket);
    }

    // 3. Request 7th - Should Fail
    let result = pool.request_socket(&url, None).await;
    assert!(result.is_err(), "Should fail when limit reached");
    assert_eq!(result.err(), Some(NetError::PreconnectMaxSocketLimit));

    // 4. Release one
    let socket = sockets.pop().unwrap();
    pool.release_socket(&url, socket, false);

    // 5. Request again - Should Succeed (Reuse)
    let result = pool.request_socket(&url, None).await;
    assert!(result.is_ok(), "Should succeed after release");
}
