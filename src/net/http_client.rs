use async_trait::async_trait;
use crate::net::{HttpClient, Response, ReqwestClient, BrowserClient};
use crate::types::error::Result;
use url::Url;
use std::sync::Arc;

pub struct SPAClient {
    static_client: Arc<ReqwestClient>,
    browser_client: Option<Arc<BrowserClient>>,
}

impl SPAClient {
    pub fn new(static_client: Arc<ReqwestClient>, browser_client: Option<Arc<BrowserClient>>) -> Self {
        Self {
            static_client,
            browser_client,
        }
    }
}

#[async_trait]
impl HttpClient for SPAClient {
    async fn fetch(&self, url: &Url) -> Result<Response> {
        if let Some(browser) = &self.browser_client {
            // For now, if browser is enabled, use it for everything
            // In a better version, we'd check extension/mime type
            browser.fetch(url).await
        } else {
            self.static_client.fetch(url).await
        }
    }
}
