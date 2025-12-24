use crate::base::neterror::NetError;
use crate::http::streamfactory::HttpStreamFactory;
use crate::http::transaction::HttpNetworkTransaction;
use http::Response;
use hyper::body::Incoming;
use std::collections::HashSet;
use std::sync::Arc;
use url::Url;

use crate::cookies::monster::CookieMonster;
use crate::urlrequest::device::Device;

pub struct URLRequestHttpJob {
    transaction: HttpNetworkTransaction,
    factory: Arc<HttpStreamFactory>,
    url: Url,
    cookie_store: Arc<CookieMonster>,
    device: Option<Device>,
    proxy_settings: Option<crate::socket::proxy::ProxySettings>,
    redirect_limit: u8,
    visited_urls: HashSet<String>,
    extra_headers: Vec<(String, String)>,
}

impl URLRequestHttpJob {
    pub fn new(
        factory: Arc<HttpStreamFactory>,
        url: Url,
        cookie_store: Arc<CookieMonster>,
    ) -> Self {
        let mut visited = HashSet::new();
        visited.insert(url.to_string());

        Self {
            transaction: HttpNetworkTransaction::new(
                factory.clone(),
                url.clone(),
                cookie_store.clone(),
            ),
            factory,
            url,
            cookie_store,
            device: None,
            proxy_settings: None,
            redirect_limit: 20, // Chromium default is 20
            visited_urls: visited,
            extra_headers: Vec::new(),
        }
    }

    pub async fn start(&mut self) -> Result<(), NetError> {
        loop {
            // Apply Headers to current transaction
            for (k, v) in &self.extra_headers {
                self.transaction.add_header(k, v)?;
            }

            // Start current transaction
            self.transaction.start().await?;

            // Check for redirect
            let should_redirect = if let Some(response) = self.transaction.get_response() {
                let status = response.status();
                if status.is_redirection() {
                    if let Some(location) = response.headers().get("Location") {
                        if let Ok(loc_str) = location.to_str() {
                            // Resolve potentially relative URL
                            self.url.join(loc_str).ok()
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            if let Some(mut new_url) = should_redirect {
                if self.redirect_limit == 0 {
                    return Err(NetError::TooManyRedirects);
                }

                // Check for redirect cycle (exact URL match)
                if !self.visited_urls.insert(new_url.to_string()) {
                    return Err(NetError::RedirectCycleDetected);
                }

                // Check Cross-Origin for Auth Stripping
                let is_cross_origin = self.url.origin() != new_url.origin();

                if is_cross_origin {
                    self.extra_headers.retain(|(k, _)| !k.eq_ignore_ascii_case("Authorization"));
                    // Strip credentials from URL (CVE-2014-1829 fix)
                    let _ = new_url.set_username("");
                    let _ = new_url.set_password(None);
                }

                self.redirect_limit -= 1;
                self.url = new_url;

                // Create new transaction for the new URL
                self.transaction = HttpNetworkTransaction::new(
                    self.factory.clone(),
                    self.url.clone(),
                    self.cookie_store.clone(),
                );

                // Restore device if set
                if let Some(device) = &self.device {
                    self.transaction.set_device(device.clone());
                }

                // Restore proxy if set
                if let Some(proxy) = &self.proxy_settings {
                    self.transaction.set_proxy(proxy.clone());
                }

                // CONTINUE LOOP
            } else {
                // Done or error
                break;
            }
        }
        Ok(())
    }

    pub fn get_response(&mut self) -> Option<&Response<Incoming>> {
        self.transaction.get_response()
    }

    pub fn set_device(&mut self, device: crate::urlrequest::device::Device) {
        self.device = Some(device.clone());
        self.transaction.set_device(device);
    }

    pub fn set_proxy(&mut self, proxy: crate::socket::proxy::ProxySettings) {
        self.proxy_settings = Some(proxy.clone());
        self.transaction.set_proxy(proxy);
    }

    pub fn add_header(&mut self, key: &str, value: &str) {
        self.extra_headers.push((key.to_string(), value.to_string()));
        // Best-effort: ignore errors for already-added headers
        let _ = self.transaction.add_header(key, value);
    }
}
