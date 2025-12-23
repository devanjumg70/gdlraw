use crate::base::neterror::NetError;
use crate::cookies::monster::CookieMonster;
use crate::http::streamfactory::HttpStreamFactory;
use crate::socket::pool::ClientSocketPool;
use crate::urlrequest::job::URLRequestHttpJob;
use http::Response;
use hyper::body::Incoming;
use once_cell::sync::Lazy;
use std::sync::Arc;
use url::Url;

// Global Context (simulating URLRequestContext)
static POOL: Lazy<Arc<ClientSocketPool>> = Lazy::new(|| Arc::new(ClientSocketPool::new()));
static FACTORY: Lazy<Arc<HttpStreamFactory>> =
    Lazy::new(|| Arc::new(HttpStreamFactory::new(POOL.clone())));
static COOKIE_STORE: Lazy<Arc<CookieMonster>> = Lazy::new(|| Arc::new(CookieMonster::new()));

pub struct URLRequest {
    job: URLRequestHttpJob,
}

impl URLRequest {
    pub fn new(url_str: &str) -> Result<Self, NetError> {
        let url = Url::parse(url_str).map_err(|_| NetError::InvalidUrl)?;

        // In real Chromium, we'd pick the job based on scheme (HttpJob, FileJob, etc)
        let job = URLRequestHttpJob::new(FACTORY.clone(), url, COOKIE_STORE.clone());

        Ok(Self { job })
    }

    pub async fn start(&mut self) -> Result<(), NetError> {
        self.job.start().await
    }

    pub fn get_response(&mut self) -> Option<&Response<Incoming>> {
        self.job.get_response()
    }

    pub fn set_device(&mut self, device: crate::urlrequest::device::Device) {
        self.job.set_device(device);
    }
}
