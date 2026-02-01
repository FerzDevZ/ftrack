use crate::types::error::Result;
use crate::types::asset::{Asset, AssetType};
use crate::net::{HttpClient, RateLimiter, robots::RobotsHandler};
use crate::core::queue::PriorityQueue;
use crate::core::deduplicator::HashDeduplicator;
use crate::parser::ContentParser;
use crate::resolver::{AssetResolver, ContentRewriter};
use crate::analyzer::{TechDetector, MetricsCollector, SecretScanner, ApiDiscovery};
use crate::storage::StorageProvider;

use std::sync::Arc;
use tracing::{info, warn, error};

pub struct WebCrawler {
    client: Arc<dyn HttpClient>,
    queue: Arc<PriorityQueue>,
    deduplicator: Arc<HashDeduplicator>,
    rate_limiter: Arc<RateLimiter>,
    robots_handler: Arc<tokio::sync::Mutex<RobotsHandler>>,
    parser: Arc<dyn ContentParser>,
    resolver: Arc<dyn AssetResolver>,
    #[allow(dead_code)]
    rewriter: Arc<dyn ContentRewriter>,
    detector: Arc<dyn TechDetector>,
    scanner: Arc<dyn SecretScanner>,
    api_discoverer: Arc<dyn ApiDiscovery>,
    metrics: Arc<dyn MetricsCollector>,
    storage: Arc<dyn StorageProvider>,
    db: Option<Arc<crate::storage::Database>>,
    tui_state: Option<Arc<tokio::sync::Mutex<crate::cli::tui::TuiState>>>,
    max_depth: usize,
    user_agent: String,
}

impl WebCrawler {
    pub fn new(
        client: Arc<dyn HttpClient>,
        queue: Arc<PriorityQueue>,
        deduplicator: Arc<HashDeduplicator>,
        rate_limiter: Arc<RateLimiter>,
        parser: Arc<dyn ContentParser>,
        resolver: Arc<dyn AssetResolver>,
        rewriter: Arc<dyn ContentRewriter>,
        detector: Arc<dyn TechDetector>,
        scanner: Arc<dyn SecretScanner>,
        api_discoverer: Arc<dyn ApiDiscovery>,
        metrics: Arc<dyn MetricsCollector>,
        storage: Arc<dyn StorageProvider>,
        db: Option<Arc<crate::storage::Database>>,
        tui_state: Option<Arc<tokio::sync::Mutex<crate::cli::tui::TuiState>>>,
        max_depth: usize,
        user_agent: String,
    ) -> Self {
        Self {
            client,
            queue,
            deduplicator,
            rate_limiter,
            robots_handler: Arc::new(tokio::sync::Mutex::new(RobotsHandler::new())),
            parser,
            resolver,
            rewriter,
            detector,
            scanner,
            api_discoverer,
            metrics,
            storage,
            db,
            tui_state,
            max_depth,
            user_agent,
        }
    }

    pub async fn crawl(&self) -> Result<()> {
        info!("Crawl started...");

        while let Some(asset) = self.queue.pop().await {
            // Check if TUI requested exit
            if let Some(tui_state) = &self.tui_state {
                let state = tui_state.lock().await;
                if state.should_quit {
                    break;
                }
            }

            if asset.depth > self.max_depth {
                continue;
            }

            if self.deduplicator.is_url_visited(&asset.url).await {
                continue;
            }

            // Update TUI current URL and statistics
            if let Some(tui_state) = &self.tui_state {
                let mut state = tui_state.lock().await;
                state.current_url = asset.url.to_string();
                state.queued_count = self.queue.len().await;
                state.add_log(format!("Crawling: {}", asset.url));
            }

            // ... robots.txt and other logic ...
            {
                let mut robots = self.robots_handler.lock().await;
                if !robots.is_allowed(&asset.url, &self.user_agent, self.client.as_ref()).await? {
                    warn!("Robots.txt disallowed: {}", asset.url);
                    if let Some(tui_state) = &self.tui_state {
                        let mut state = tui_state.lock().await;
                        state.add_log(format!("Disallowed: {}", asset.url));
                    }
                    continue;
                }
            }

            let host = asset.url.host_str().unwrap_or("unknown");
            let _permit = self.rate_limiter.acquire(host).await;

            match self.client.fetch(&asset.url).await {
                Ok(resp) => {
                    self.metrics.record_request(resp.content.len());
                    self.deduplicator.mark_url_visited(asset.url.clone()).await;

                    let local_path = self.resolver.resolve_local_path(&asset.url);
                    let content_str = String::from_utf8_lossy(&resp.content);

                    let tech = self.detector.detect(&asset, &content_str);
                    if !tech.is_empty() {
                        if let Some(db) = &self.db {
                            let _ = db.save_tech_stack(&asset.url, tech.clone()).await;
                        }
                        if let Some(tui_state) = &self.tui_state {
                            let mut state = tui_state.lock().await;
                            for t in &tech {
                                if !state.tech_detected.contains(t) {
                                    state.tech_detected.push(t.clone());
                                }
                            }
                        }
                    }

                    // V3: Secret Scanning
                    let leaks = self.scanner.scan(asset.url.as_str(), &content_str);
                    if !leaks.is_empty() {
                        for leak in leaks {
                            if let Some(db) = &self.db {
                                let _ = db.save_leak(leak.clone()).await;
                            }
                            if let Some(tui_state) = &self.tui_state {
                                let mut state = tui_state.lock().await;
                                state.leaks_detected.push(leak);
                            }
                        }
                    }

                    // V3: API Discovery
                    let endpoints = self.api_discoverer.discover(asset.url.as_str(), &content_str);
                    if !endpoints.is_empty() {
                        for endpoint in endpoints {
                            if let Some(db) = &self.db {
                                let _ = db.save_endpoint(endpoint.clone()).await;
                            }
                            if let Some(tui_state) = &self.tui_state {
                                let mut state = tui_state.lock().await;
                                state.api_endpoints.push(endpoint);
                            }
                        }
                    }

                    if matches!(asset.asset_type, AssetType::Html) {
                        let links = self.parser.extract_links(&content_str, &asset.url)?;
                        for link in links {
                            let new_asset = Asset::new(link.clone(), "".into(), AssetType::Html, asset.depth + 1);
                            self.queue.push(new_asset).await;
                        }

                        let sub_assets = self.parser.extract_assets(&content_str, &asset.url)?;
                        for sa in sub_assets {
                            let mut sa_final = sa;
                            sa_final.depth = asset.depth + 1;
                            self.queue.push(sa_final).await;
                        }
                    }

                    self.storage.save(&local_path, &resp.content).await?;

                    // Update TUI statistics
                    if let Some(tui_state) = &self.tui_state {
                        let mut state = tui_state.lock().await;
                        state.visited_count += 1;
                    }
                }
                Err(e) => {
                    error!("Failed to fetch {}: {}", asset.url, e);
                    if let Some(tui_state) = &self.tui_state {
                        let mut state = tui_state.lock().await;
                        state.error_count += 1;
                        state.add_log(format!("Error: {}", e));
                    }
                }
            }
        }

        info!("Crawl finished.");
        if self.tui_state.is_none() {
            println!("{}", self.metrics.report());
        } else if let Some(tui_state) = &self.tui_state {
            let mut state = tui_state.lock().await;
            state.should_quit = true;
        }
        Ok(())
    }
}
