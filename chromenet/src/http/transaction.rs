use crate::base::loadstate::LoadState;
use crate::base::neterror::NetError;
use crate::http::orderedheaders::OrderedHeaderMap;
use crate::http::retry::{calculate_backoff, RetryConfig, RetryReason};
use crate::http::streamfactory::{HttpStream, HttpStreamFactory};
use crate::http::H2Settings;
use http::{Request, Response, Version};
use hyper::body::Incoming;
use std::sync::Arc;
use url::Url;

use crate::cookies::monster::CookieMonster;
use crate::urlrequest::device::Device;

/// Internal state machine states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Idle,
    CreateStream,
    SendRequest,
    ReadHeaders,
    Done,
}

impl State {
    /// Map internal state to public LoadState.
    fn to_load_state(self) -> LoadState {
        match self {
            State::Idle => LoadState::Idle,
            State::CreateStream => LoadState::Connecting,
            State::SendRequest => LoadState::SendingRequest,
            State::ReadHeaders => LoadState::WaitingForResponse,
            State::Done => LoadState::Idle,
        }
    }
}

pub struct HttpNetworkTransaction {
    factory: Arc<HttpStreamFactory>,
    url: Url,
    state: State,
    stream: Option<HttpStream>,
    response: Option<Response<Incoming>>,
    request_headers: OrderedHeaderMap,
    device: Option<Device>,
    h2_settings: Option<H2Settings>,
    cookie_store: Arc<CookieMonster>,
    proxy_settings: Option<crate::socket::proxy::ProxySettings>,
    retry_config: RetryConfig,
    retry_attempts: usize,
}

impl HttpNetworkTransaction {
    pub fn new(
        factory: Arc<HttpStreamFactory>,
        url: Url,
        cookie_store: Arc<CookieMonster>,
    ) -> Self {
        Self {
            factory,
            url,
            state: State::Idle,
            stream: None,
            response: None,
            request_headers: OrderedHeaderMap::default(),
            device: None,
            h2_settings: None,
            cookie_store,
            proxy_settings: None,
            retry_config: RetryConfig::default(),
            retry_attempts: 0,
        }
    }

    /// Set custom retry configuration.
    pub fn set_retry_config(&mut self, config: RetryConfig) {
        self.retry_config = config;
    }

    /// Get the current load state (for progress reporting).
    pub fn get_load_state(&self) -> LoadState {
        self.state.to_load_state()
    }

    pub fn set_device(&mut self, device: Device) {
        self.device = Some(device);
    }

    pub fn set_proxy(&mut self, proxy: crate::socket::proxy::ProxySettings) {
        self.proxy_settings = Some(proxy);
    }

    /// Set HTTP/2 SETTINGS for fingerprinting.
    pub fn set_h2_settings(&mut self, settings: H2Settings) {
        self.h2_settings = Some(settings);
    }

    pub fn set_headers(&mut self, headers: OrderedHeaderMap) {
        self.request_headers = headers;
    }

    /// Add a header to the request.
    /// Returns an error if the header name or value is invalid.
    pub fn add_header(&mut self, key: &str, value: &str) -> Result<(), NetError> {
        self.request_headers
            .insert(key, value)
            .map_err(|_| NetError::InvalidUrl)
    }

    /// Start the transaction with automatic retry on connection failures.
    pub async fn start(&mut self) -> Result<(), NetError> {
        self.state = State::CreateStream;
        self.retry_attempts = 0;

        loop {
            match self.do_loop().await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    // Check if this error is retryable
                    if let Some(_reason) = RetryReason::from_error(&e) {
                        if self.retry_attempts < self.retry_config.max_attempts {
                            let delay = calculate_backoff(self.retry_attempts, &self.retry_config);
                            self.retry_attempts += 1;

                            // Reset state for retry
                            self.state = State::CreateStream;
                            self.stream = None;
                            self.response = None;

                            // Wait with exponential backoff
                            tokio::time::sleep(delay).await;
                            continue;
                        }
                    }
                    return Err(e);
                }
            }
        }
    }

    async fn do_loop(&mut self) -> Result<(), NetError> {
        loop {
            match self.state {
                State::Idle => {
                    return Ok(());
                }
                State::CreateStream => {
                    self.stream = Some(
                        self.factory
                            .create_stream(
                                &self.url,
                                self.proxy_settings.as_ref(),
                                self.h2_settings.as_ref(),
                            )
                            .await?,
                    );
                    self.state = State::SendRequest;
                }
                State::SendRequest => {
                    let is_h2 = self.stream.as_ref().map(|s| s.is_h2()).unwrap_or(false);

                    // Host header (Only for H1)
                    if !is_h2 && self.request_headers.get("Host").is_none() {
                        let host = self.url.host_str().ok_or(NetError::InvalidUrl)?;
                        self.request_headers
                            .insert("Host", host)
                            .map_err(|_| NetError::InvalidUrl)?;
                    }

                    // Cookie header: Query the cookie store
                    let cookies = self.cookie_store.get_cookies_for_url(&self.url);
                    if !cookies.is_empty() {
                        // Format cookies as "name=value; name2=value2"
                        // Chromium sorts by path length (longest first) and creation time (oldest first).
                        // get_cookies_for_url already returns them sorted correctly.
                        let cookie_value = cookies
                            .iter()
                            .map(|c| format!("{}={}", c.name, c.value))
                            .collect::<Vec<_>>()
                            .join("; ");

                        self.request_headers
                            .insert("Cookie", &cookie_value)
                            .map_err(|_| NetError::InvalidUrl)?;
                    }

                    // Build request
                    let version = if is_h2 {
                        Version::HTTP_2
                    } else {
                        Version::HTTP_11
                    };
                    let builder = Request::builder().uri(self.url.as_str()).version(version);

                    let headers_map = self.request_headers.clone().to_header_map();

                    let mut req = builder
                        .body(http_body_util::Empty::<bytes::Bytes>::new())
                        .map_err(|_| NetError::InvalidUrl)?;

                    *req.headers_mut() = headers_map;

                    if let Some(stream) = self.stream.as_mut() {
                        match stream.send_request(req).await {
                            Ok(resp) => {
                                // Process Set-Cookie headers
                                for val in resp.headers().get_all(http::header::SET_COOKIE) {
                                    if let Ok(s) = val.to_str() {
                                        self.cookie_store.parse_and_save_cookie(&self.url, s);
                                    }
                                }

                                self.response = Some(resp);
                                self.state = State::ReadHeaders;
                            }
                            Err(e) => {
                                // Retry on reused socket failure
                                if stream.is_reused() {
                                    eprintln!(
                                        "Socket reuse failed. Retrying with fresh connection."
                                    );
                                    self.factory.report_failure(&self.url);
                                    self.stream = None;
                                    self.state = State::CreateStream;
                                } else {
                                    return Err(e);
                                }
                            }
                        }
                    } else {
                        return Err(NetError::ConnectionClosed);
                    }
                }
                State::ReadHeaders => {
                    self.state = State::Done;
                    return Ok(());
                }
                State::Done => return Ok(()),
            }
        }
    }

    pub fn get_response(&mut self) -> Option<&Response<Incoming>> {
        self.response.as_ref()
    }

    /// Take ownership of the response, converting to HttpResponse.
    /// Can only be called once - subsequent calls return None.
    pub fn take_response(&mut self) -> Option<crate::http::response::HttpResponse> {
        self.response
            .take()
            .map(crate::http::response::HttpResponse::from_hyper)
    }
}
