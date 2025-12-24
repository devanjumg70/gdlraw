use crate::base::neterror::NetError;
use crate::cookies::monster::CookieMonster;
use crate::http::streamfactory::HttpStreamFactory;
use crate::socket::pool::ClientSocketPool;
use crate::urlrequest::job::URLRequestHttpJob;
use hyper::body::Incoming;
use std::sync::{Arc, OnceLock};
use url::Url;

// Global singletons using std::sync::OnceLock (replaces once_cell)
static POOL: OnceLock<Arc<ClientSocketPool>> = OnceLock::new();
static FACTORY: OnceLock<Arc<HttpStreamFactory>> = OnceLock::new();
static COOKIE_STORE: OnceLock<Arc<CookieMonster>> = OnceLock::new();

fn get_pool() -> &'static Arc<ClientSocketPool> {
    POOL.get_or_init(|| Arc::new(ClientSocketPool::new()))
}

fn get_factory() -> &'static Arc<HttpStreamFactory> {
    FACTORY.get_or_init(|| Arc::new(HttpStreamFactory::new(get_pool().clone())))
}

fn get_cookie_store() -> &'static Arc<CookieMonster> {
    COOKIE_STORE.get_or_init(|| Arc::new(CookieMonster::new()))
}

pub struct URLRequest {
    job: URLRequestHttpJob,
}

impl URLRequest {
    pub fn new(url_str: &str) -> Result<Self, NetError> {
        let url = Url::parse(url_str).map_err(|_| NetError::InvalidUrl)?;

        // In real Chromium, we'd pick the job based on scheme (HttpJob, FileJob, etc)
        let job = URLRequestHttpJob::new(get_factory().clone(), url, get_cookie_store().clone());

        Ok(Self { job })
    }

    pub async fn start(&mut self) -> Result<(), NetError> {
        self.job.start().await
    }

    pub fn get_response(&mut self) -> Option<&http::Response<Incoming>> {
        self.job.get_response()
    }

    /// Take ownership of the response with body.
    pub fn take_response(&mut self) -> Option<crate::http::HttpResponse> {
        self.job.take_response()
    }

    pub fn set_device(&mut self, device: crate::urlrequest::device::Device) {
        self.job.set_device(device);
    }

    pub fn set_proxy(&mut self, proxy: crate::socket::proxy::ProxySettings) {
        self.job.set_proxy(proxy);
    }

    pub fn add_header(&mut self, key: &str, value: &str) {
        self.job.add_header(key, value);
    }

    /// Set the HTTP method.
    pub fn set_method(&mut self, method: http::Method) {
        self.job.set_method(method);
    }

    /// Set the request body.
    pub fn set_body(&mut self, body: impl Into<crate::http::RequestBody>) {
        self.job.set_body(body);
    }

    /// Create a POST request.
    pub fn post(url_str: &str) -> Result<Self, NetError> {
        let mut req = Self::new(url_str)?;
        req.set_method(http::Method::POST);
        Ok(req)
    }

    /// Create a PUT request.
    pub fn put(url_str: &str) -> Result<Self, NetError> {
        let mut req = Self::new(url_str)?;
        req.set_method(http::Method::PUT);
        Ok(req)
    }
}
