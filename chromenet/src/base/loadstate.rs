/// The current state of a URLRequest.
/// This roughly matches net/base/load_states.h
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LoadState {
    /// The request is idle.
    #[default]
    Idle,

    /// Waiting for a socket from the pool.
    WaitingForStalledSocketPool,

    /// Waiting for an available socket.
    WaitingForAvailableSocket,

    /// Waiting for the delegate to run a job.
    WaitingForDelegate,

    /// Waiting for the cache lock.
    WaitingForCache,

    /// Waiting for the app cache.
    #[deprecated(note = "Obsolete in Chromium")]
    ObsoleteWaitingForAppCache,

    /// Downloading the PAC script.
    DownloadingPacFile,

    /// Resolving the proxy.
    ResolvingProxyForUrl,

    /// Resolving the host in PAC file.
    ResolvingHostInPacFile,

    /// Establishing proxy tunnel.
    EstablishingProxyTunnel,

    /// Resolving the host.
    ResolvingHost,

    /// Connecting to the host (TCP handshake).
    Connecting,

    /// Establishing an SSL connection.
    SslHandshake,

    /// Sending the HTTP request.
    SendingRequest,

    /// Waiting for the server response (TTFB).
    WaitingForResponse,

    /// Reading the response body.
    ReadingResponse,
}
