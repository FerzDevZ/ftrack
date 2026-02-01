use async_trait::async_trait;
use crate::types::error::{ArchivError, Result};
use url::Url;
use bytes::Bytes;
use reqwest::Client;
use std::collections::HashMap;
use std::time::Duration;

#[async_trait]
pub trait HttpClient: Send + Sync {
    async fn fetch(&self, url: &Url) -> Result<Response>;
}

pub struct Response {
    pub content: Bytes,
    #[allow(dead_code)]
    pub status: u16,
    #[allow(dead_code)]
    pub headers: HashMap<String, String>,
}

pub struct ReqwestClient {
    client: Client,
}

impl ReqwestClient {
    pub fn new(user_agent: &str, timeout: Duration) -> Result<Self> {
        let client = Client::builder()
            .user_agent(user_agent)
            .timeout(timeout)
            .cookie_store(true)
            .build()?;
        Ok(Self { client })
    }
}

#[async_trait]
impl HttpClient for ReqwestClient {
    async fn fetch(&self, url: &Url) -> Result<Response> {
        let resp = self.client.get(url.clone()).send().await?;
        let status = resp.status().as_u16();
        
        if !resp.status().is_success() {
            return Err(ArchivError::Network(resp.error_for_status().unwrap_err()));
        }

        let headers = resp.headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or_default().to_string()))
            .collect();
            
        let content = resp.bytes().await?;
        
        Ok(Response {
            content,
            status,
            headers,
        })
    }
}

pub mod http_client;
pub mod rate_limiter;
pub mod robots;
pub mod browser;

pub use rate_limiter::RateLimiter;
pub use browser::BrowserClient;
