use crate::base::neterror::NetError;
use crate::http::streamfactory::HttpStreamFactory;
use crate::http::transaction::HttpNetworkTransaction;
use crate::http::RequestBody;
use http::{Method, Response};
use hyper::body::Incoming;
use std::collections::HashSet;
use std::sync::Arc;
use url::Url;

use crate::cookies::monster::CookieMonster;
use crate::urlrequest::device::Device;

/// Compute the method to use after a redirect.
/// Mirrors Chromium's ComputeMethodForRedirect in redirect_info.cc.
/// Per RFC 7231:
/// - 303 redirects convert all methods except HEAD to GET
/// - 301/302 redirects convert POST to GET (historical behavior)
/// - 307/308 preserve the original method
fn compute_method_for_redirect(method: &Method, status_code: u16) -> Method {
    if (status_code == 303 && method != Method::HEAD)
        || ((status_code == 301 || status_code == 302) && method == Method::POST)
    {
        Method::GET
    } else {
        method.clone()
    }
}

pub struct URLRequestHttpJob {
    transaction: HttpNetworkTransaction,
    factory: Arc<HttpStreamFactory>,
    url: Url,
    method: Method,
    body: RequestBody,
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
            method: Method::GET,
            body: RequestBody::default(),
            cookie_store,
            device: None,
            proxy_settings: None,
            redirect_limit: 20, // Chromium default is 20
            visited_urls: visited,
            extra_headers: Vec::new(),
        }
    }

    /// Set the HTTP method.
    pub fn set_method(&mut self, method: Method) {
        self.method = method;
    }

    /// Set the request body.
    pub fn set_body(&mut self, body: impl Into<RequestBody>) {
        self.body = body.into();
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

                // Get status code for method computation
                let status_code = self
                    .transaction
                    .get_response()
                    .map(|r| r.status().as_u16())
                    .unwrap_or(0);

                // Compute new method per RFC 7231 (Chromium's ComputeMethodForRedirect)
                let new_method = compute_method_for_redirect(&self.method, status_code);

                // If method changed to GET, clear the body
                if new_method != self.method && new_method == Method::GET {
                    self.body = RequestBody::default();
                }
                self.method = new_method;

                // Check for redirect cycle (exact URL match)
                if !self.visited_urls.insert(new_url.to_string()) {
                    return Err(NetError::RedirectCycleDetected);
                }

                // Check Cross-Origin for Auth Stripping
                let is_cross_origin = self.url.origin() != new_url.origin();

                if is_cross_origin {
                    self.extra_headers
                        .retain(|(k, _)| !k.eq_ignore_ascii_case("Authorization"));
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

    /// Take ownership of the response with body.
    pub fn take_response(&mut self) -> Option<crate::http::HttpResponse> {
        self.transaction.take_response()
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
        self.extra_headers
            .push((key.to_string(), value.to_string()));
        // Best-effort: ignore errors for already-added headers
        let _ = self.transaction.add_header(key, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_301_post_becomes_get() {
        let result = compute_method_for_redirect(&Method::POST, 301);
        assert_eq!(result, Method::GET);
    }

    #[test]
    fn test_302_post_becomes_get() {
        let result = compute_method_for_redirect(&Method::POST, 302);
        assert_eq!(result, Method::GET);
    }

    #[test]
    fn test_303_any_becomes_get() {
        assert_eq!(compute_method_for_redirect(&Method::POST, 303), Method::GET);
        assert_eq!(compute_method_for_redirect(&Method::PUT, 303), Method::GET);
        assert_eq!(
            compute_method_for_redirect(&Method::DELETE, 303),
            Method::GET
        );
    }

    #[test]
    fn test_303_head_stays_head() {
        let result = compute_method_for_redirect(&Method::HEAD, 303);
        assert_eq!(result, Method::HEAD);
    }

    #[test]
    fn test_307_preserves_method() {
        assert_eq!(
            compute_method_for_redirect(&Method::POST, 307),
            Method::POST
        );
        assert_eq!(compute_method_for_redirect(&Method::PUT, 307), Method::PUT);
        assert_eq!(
            compute_method_for_redirect(&Method::DELETE, 307),
            Method::DELETE
        );
    }

    #[test]
    fn test_308_preserves_method() {
        assert_eq!(
            compute_method_for_redirect(&Method::POST, 308),
            Method::POST
        );
        assert_eq!(compute_method_for_redirect(&Method::PUT, 308), Method::PUT);
    }

    #[test]
    fn test_301_get_stays_get() {
        let result = compute_method_for_redirect(&Method::GET, 301);
        assert_eq!(result, Method::GET);
    }

    #[test]
    fn test_301_put_stays_put() {
        // PUT is not POST, so 301 preserves it
        let result = compute_method_for_redirect(&Method::PUT, 301);
        assert_eq!(result, Method::PUT);
    }

    #[test]
    fn test_200_no_redirect() {
        // Non-redirect status code
        let result = compute_method_for_redirect(&Method::POST, 200);
        assert_eq!(result, Method::POST);
    }
}
