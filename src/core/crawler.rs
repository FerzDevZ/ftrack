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
use std::time::Duration;
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

    pub async fn crawl(self: Arc<Self>) -> Result<()> {
        info!("Crawl started (Parallel Engine)...");
        let mut join_set = tokio::task::JoinSet::new();
        
        // We use a semaphore to limit global concurrency
        // This prevents spawning too many tasks at once
        let global_concurrency = self.rate_limiter.max_concurrent_requests();
        let semaphore = Arc::new(tokio::sync::Semaphore::new(global_concurrency));

        loop {
            // Check if TUI requested exit
            if let Some(tui_state) = &self.tui_state {
                let state = tui_state.lock().await;
                if state.should_quit {
                    break;
                }
            }

            // Pop an asset if we have capacity
            let permit = semaphore.clone().try_acquire_owned();
            
            match permit {
                Ok(permit) => {
                    if let Some(asset) = self.queue.pop().await {
                        if asset.depth > self.max_depth || self.deduplicator.is_url_visited(&asset.url).await {
                             // Permit will be dropped, capacity restored
                             continue;
                        }

                        let crawler = self.clone();
                        join_set.spawn(async move {
                            let _permit = permit; // Hold permit until done
                            let _ = crawler.process_asset(asset).await;
                        });
                    } else {
                        // Queue empty, but check if workers are still running
                        if join_set.is_empty() {
                            break;
                        }
                        // Wait for any worker to finish to see if they add more to queue
                        tokio::select! {
                             _ = join_set.join_next() => {},
                             _ = tokio::time::sleep(Duration::from_millis(100)) => {}
                        }
                    }
                }
                Err(_) => {
                    // At capacity, wait for a worker to finish
                    if let Some(res) = join_set.join_next().await {
                        if let Err(e) = res {
                            error!("Worker task failed: {}", e);
                        }
                    }
                }
            }
        }

        // Wait for remaining tasks
        while let Some(res) = join_set.join_next().await {
            if let Err(e) = res {
                error!("Worker task failed during shutdown: {}", e);
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

    async fn process_asset(&self, asset: Asset) -> Result<()> {
        // Update TUI current URL and statistics
        if let Some(tui_state) = &self.tui_state {
            let mut state = tui_state.lock().await;
            state.current_url = asset.url.to_string();
            state.queued_count = self.queue.len().await;
            state.add_log(format!("Processing: {}", asset.url));
        }

        {
            let mut robots = self.robots_handler.lock().await;
            if !robots.is_allowed(&asset.url, &self.user_agent, self.client.as_ref()).await? {
                warn!("Robots.txt disallowed: {}", asset.url);
                if let Some(tui_state) = &self.tui_state {
                    let mut state = tui_state.lock().await;
                    state.add_log(format!("Disallowed: {}", asset.url));
                }
                return Ok(());
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
                            state.leaks_detected.push(leak.clone());
                            state.add_log(format!("[ALERT] Found {}: {}", leak.kind, leak.value));
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
                            state.api_endpoints.push(endpoint.clone());
                            state.add_log(format!("[API] Discovered: {}", endpoint.url));
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
                    state.add_log(format!("Saved: {}", local_path.display()));
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
        Ok(())
    }
}
