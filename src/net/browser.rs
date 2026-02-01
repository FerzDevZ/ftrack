use async_trait::async_trait;
use crate::net::{HttpClient, Response};
use crate::types::error::{ArchivError, Result};
use chromiumoxide::browser::{Browser, BrowserConfig};
use futures_util::StreamExt;
use url::Url;
use std::collections::HashMap;
use bytes::Bytes;

pub struct BrowserClient {
    browser: tokio::sync::Mutex<Browser>,
}

impl BrowserClient {
    pub async fn new() -> Result<(Self, tokio::task::JoinHandle<()>)> {
        let (browser, mut handler) = Browser::launch(
            BrowserConfig::builder()
                // .viewport(...) // Default viewport
                .with_head() // Useful for debugging
                .build()
                .map_err(|e| ArchivError::Unknown(e.to_string()))?,
        )
        .await
        .map_err(|e| ArchivError::Unknown(e.to_string()))?;

        let handle = tokio::spawn(async move {
            while let Some(h) = handler.next().await {
                if h.is_err() {
                    break;
                }
            }
        });
        Ok((Self { browser: tokio::sync::Mutex::new(browser) }, handle))
    }

    pub async fn close(&self) {
        let mut browser = self.browser.lock().await;
        let _ = browser.close().await;
    }
}

#[async_trait]
impl HttpClient for BrowserClient {
    async fn fetch(&self, url: &Url) -> Result<Response> {
        let browser = self.browser.lock().await;
        let page = browser
            .new_page(url.clone())
            .await
            .map_err(|e| ArchivError::Browser(e.to_string()))?;

        // Wait for dynamic content to load (simple timeout/network idle proxy)
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

        let content = page
            .content()
            .await
            .map_err(|e| ArchivError::Browser(e.to_string()))?;

        Ok(Response {
            content: Bytes::from(content),
            status: 200,
            headers: HashMap::new(), // Chromiumoxide doesn't easily expose all final headers in this way
        })
    }
}
