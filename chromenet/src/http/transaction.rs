use crate::base::neterror::NetError;
use crate::http::orderedheaders::OrderedHeaderMap;
use crate::http::streamfactory::{HttpStream, HttpStreamFactory};
use http::{Request, Response, Version};
use hyper::body::Incoming;
use std::sync::Arc;
use url::Url;

enum State {
    CreateStream,
    SendRequest,
    ReadHeaders,
    Done,
}

use crate::cookies::monster::CookieMonster;
use crate::urlrequest::device::Device;

pub struct HttpNetworkTransaction {
    factory: Arc<HttpStreamFactory>,
    url: Url,
    state: State,
    stream: Option<HttpStream>,
    response: Option<Response<Incoming>>,
    request_headers: OrderedHeaderMap,
    device: Option<Device>,
    cookie_store: Arc<CookieMonster>,
    proxy_settings: Option<crate::socket::proxy::ProxySettings>,
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
            state: State::CreateStream,
            stream: None,
            response: None,
            request_headers: OrderedHeaderMap::new(),
            device: None,
            cookie_store,
            proxy_settings: None,
        }
    }

    pub fn set_device(&mut self, device: Device) {
        self.device = Some(device);
    }

    pub fn set_proxy(&mut self, proxy: crate::socket::proxy::ProxySettings) {
        self.proxy_settings = Some(proxy);
    }

    pub fn set_headers(&mut self, headers: OrderedHeaderMap) {
        self.request_headers = headers;
    }

    pub async fn start(&mut self) -> Result<(), NetError> {
        self.state = State::CreateStream;
        self.do_loop().await
    }

    async fn do_loop(&mut self) -> Result<(), NetError> {
        loop {
            match self.state {
                State::CreateStream => {
                    self.stream = Some(
                        self.factory.create_stream(&self.url, self.proxy_settings.as_ref()).await?,
                    );
                    self.state = State::SendRequest;
                }
                State::SendRequest => {
                    // 1. Build Standard Headers (Host, Connection, User-Agent)
                    // Chromium logic: explicit order.

                    let is_h2 = self.stream.as_ref().map(|s| s.is_h2()).unwrap_or(false);

                    // Host (Only for H1, or rely on Hyper for H2 authority)
                    if !is_h2 && self.request_headers.get("Host").is_none() {
                        let host = self.url.host_str().ok_or(NetError::InvalidUrl)?;
                        self.request_headers
                            .insert("Host", host)
                            .map_err(|_| NetError::InvalidUrl)?;
                    }

                    // ... User-Agent logic remains ...

                    // 139: Request Builder
                    let version = if is_h2 { Version::HTTP_2 } else { Version::HTTP_11 };
                    let builder = Request::builder().uri(self.url.as_str()).version(version);

                    // Hydrate headers from OrderedHeaderMap
                    // Note: This relies on hyper/http preserving order of append.
                    let headers_map = self.request_headers.clone().to_header_map();

                    let mut req = builder
                        .body(http_body_util::Empty::<bytes::Bytes>::new())
                        .map_err(|_| NetError::InvalidUrl)?;

                    *req.headers_mut() = headers_map;

                    if let Some(stream) = self.stream.as_mut() {
                        match stream.send_request(req).await {
                            Ok(resp) => {
                                // Process Set-Cookie
                                for val in resp.headers().get_all(http::header::SET_COOKIE) {
                                    if let Ok(s) = val.to_str() {
                                        self.cookie_store.parse_and_save_cookie(&self.url, s);
                                    }
                                }

                                self.response = Some(resp);
                                self.state = State::ReadHeaders;
                            }
                            Err(e) => {
                                // Retry Logic
                                if stream.is_reused() {
                                    eprintln!(
                                        "Socket reuse failed. Retrying with fresh connection."
                                    );
                                    self.factory.report_failure(&self.url);
                                    self.stream = None;
                                    self.state = State::CreateStream;
                                    // Implicit continue of loop
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
                    // Start() usually finishes when headers are available
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
}
