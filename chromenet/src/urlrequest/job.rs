use crate::base::neterror::NetError;
use crate::http::streamfactory::HttpStreamFactory;
use crate::http::transaction::HttpNetworkTransaction;
use http::Response;
use hyper::body::Incoming;
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
    redirect_limit: u8,
}

impl URLRequestHttpJob {
    pub fn new(
        factory: Arc<HttpStreamFactory>,
        url: Url,
        cookie_store: Arc<CookieMonster>,
    ) -> Self {
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
            redirect_limit: 20, // Chromium default is 20
        }
    }

    pub async fn start(&mut self) -> Result<(), NetError> {
        loop {
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

            if let Some(new_url) = should_redirect {
                if self.redirect_limit == 0 {
                    return Err(NetError::TooManyRedirects);
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
}
