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
        }
    }

    pub fn set_device(&mut self, device: Device) {
        self.device = Some(device);
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
                    self.stream = Some(self.factory.request_stream(&self.url).await?);
                    self.state = State::SendRequest;
                }
                State::SendRequest => {
                    // 1. Build Standard Headers (Host, Connection, User-Agent)
                    // Chromium logic: explicit order.

                    // Host
                    if self.request_headers.get("Host").is_none() {
                        let host = self.url.host_str().ok_or(NetError::InvalidUrl)?;
                        self.request_headers
                            .insert("Host", host)
                            .map_err(|_| NetError::InvalidUrl)?;
                    }

                    // User-Agent & Client Hints (Device Emulation)
                    if let Some(device) = &self.device {
                        self.request_headers
                            .insert("User-Agent", device.user_agent)
                            .map_err(|_| NetError::InvalidUrl)?;

                        if let Some(meta) = &device.user_agent_metadata {
                            // Sec-Ch-Ua
                            // Format: "Google Chrome";v="117", "Not;A=Brand";v="8", "Chromium";v="117"
                            // This is complex to reconstruct perfectly without matching exact version.
                            // For now, we use a static placeholder or basic construction.
                            // Chromium's `EmulatedDevices.ts` doesn't give full Sec-Ch-Ua string, just metadata.
                            // We'll construct a plausible one.
                            let brand_version = match meta.platform {
                                 "Android" => "\"Google Chrome\";v=\"117\", \"Not;A=Brand\";v=\"8\", \"Chromium\";v=\"117\"",
                                 _ => "\"Google Chrome\";v=\"117\", \"Not;A=Brand\";v=\"8\", \"Chromium\";v=\"117\"", 
                             };
                            self.request_headers
                                .insert("Sec-Ch-Ua", brand_version)
                                .map_err(|_| NetError::InvalidUrl)?;

                            self.request_headers
                                .insert("Sec-Ch-Ua-Mobile", if meta.mobile { "?1" } else { "?0" })
                                .map_err(|_| NetError::InvalidUrl)?;
                            self.request_headers
                                .insert("Sec-Ch-Ua-Platform", &format!("\"{}\"", meta.platform))
                                .map_err(|_| NetError::InvalidUrl)?;
                            // TODO: platform-version, model, arch, bitness, full-version-list (if high entropy)
                        }
                    } else if self.request_headers.get("User-Agent").is_none() {
                        // Default UA if no device set
                        self.request_headers
                            .insert("User-Agent", "Mozilla/5.0 (ChromiumNet)")
                            .map_err(|_| NetError::InvalidUrl)?;
                    }

                    // 2. Cookie Injection
                    let cookies = self.cookie_store.get_cookies_for_url(&self.url);
                    if !cookies.is_empty() {
                        let cookie_val = cookies
                            .iter()
                            .map(|c| format!("{}={}", c.name, c.value))
                            .collect::<Vec<_>>()
                            .join("; ");
                        self.request_headers
                            .insert("Cookie", &cookie_val)
                            .map_err(|_| NetError::InvalidUrl)?;
                    }

                    // 3. Convert to http::Request
                    let builder =
                        Request::builder().uri(self.url.as_str()).version(Version::HTTP_11);

                    // Hydrate headers from OrderedHeaderMap
                    // Note: This relies on hyper/http preserving order of append.
                    let headers_map = self.request_headers.clone().to_header_map();

                    let mut req = builder
                        .body(http_body_util::Empty::<bytes::Bytes>::new())
                        .map_err(|_| NetError::InvalidUrl)?;

                    *req.headers_mut() = headers_map;

                    if let Some(stream) = self.stream.as_mut() {
                        let resp = stream.send_request(req).await?;

                        // Process Set-Cookie
                        // Note: hyper::HeaderMap doesn't easily give multiple Set-Cookie lines if using get()
                        // getAll() equivalents in hyper 1.0 needed?
                        // Actually resp.headers().get_all(SET_COOKIE) returns iterator.
                        for val in resp.headers().get_all(http::header::SET_COOKIE) {
                            if let Ok(s) = val.to_str() {
                                // TODO: Parse cookie properly
                                // For now, just a dummy parsing or simple one to prove persistence in monster
                                // println!("Received Set-Cookie: {}", s);
                                self.cookie_store.parse_and_save_cookie(&self.url, s);
                            }
                        }

                        self.response = Some(resp);
                        self.state = State::ReadHeaders;
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
