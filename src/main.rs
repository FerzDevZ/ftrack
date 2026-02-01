mod cli;
mod core;
mod net;
mod parser;
mod resolver;
mod analyzer;
mod storage;
mod types;

use clap::Parser;
use cli::args::Args;
use cli::config::AppConfig;
use core::crawler::WebCrawler;
use core::queue::PriorityQueue;
use core::deduplicator::HashDeduplicator;
use net::{ReqwestClient, BrowserClient, http_client::SPAClient, rate_limiter::RateLimiter};
use resolver::{asset::DefaultAssetResolver, rewrite::DefaultContentRewriter};
use analyzer::{
    tech_detect::DefaultTechDetector, 
    metrics::DefaultMetricsCollector,
    secret_scanner::DefaultSecretScanner,
    api_discover::DefaultApiDiscoverer,
};
use storage::{
    filesystem::FileSystemStorage,
    warc::WarcStorage,
    monolith::MonolithStorage,
    StorageProvider,
};
use types::asset::{Asset, AssetType};

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, error, Level};
use tracing_subscriber::FmtSubscriber;
use url::Url;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Parse CLI arguments
    let args = Args::parse();

    // 2. Load Config
    let config = if let Some(cfg_path) = args.config {
        AppConfig::from_file(&cfg_path)?
    } else {
        AppConfig::default()
    };

    // 3. Initialize tracing (logging)
    let log_level = match args.log_level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    info!("Starting Web Archiver & Analyzer...");

    // 4. Initialize Components
    let url_str = if !args.url.contains("://") {
        format!("https://{}", args.url)
    } else {
        args.url.clone()
    };
    let target_url = Url::parse(&url_str)?;
    
    // Smart default output: use domain name if output is "archive"
    let output_dir = if args.output.to_string_lossy() == "archive" {
        let host = target_url.host_str().unwrap_or("archive");
        PathBuf::from(host)
    } else {
        args.output
    };

    info!("Output directory: {:?}", output_dir);
    // Ensure output dir exists
    std::fs::create_dir_all(&output_dir)?;

    // V2: Initialize Database for Incremental Archiving
    let db_path = output_dir.join("crawl_state.db");
    let db = Arc::new(storage::Database::open(db_path).await?);

    let static_client = Arc::new(ReqwestClient::new(&config.crawler.user_agent, Duration::from_secs(30))?);
    
    let mut browser_client = None;
    let mut browser_handle = None;
    if args.spa {
        info!("Initializing headless browser for SPA support...");
        let (bc, handle) = BrowserClient::new().await?;
        browser_client = Some(Arc::new(bc));
        browser_handle = Some(handle);
    }
    
    let client = Arc::new(SPAClient::new(static_client, browser_client.clone()));
    let queue = Arc::new(PriorityQueue::with_db(db.clone()));
    let deduplicator = Arc::new(HashDeduplicator::with_db(db.clone()));
    let rate_limiter = Arc::new(RateLimiter::new(config.crawler.concurrent_requests, config.crawler.rate_limit_delay_ms));
    
    // Parsers could be combined into a multi-parser, but for now we use the main types
    let parser = Arc::new(parser::html::HtmlParser); 
    let resolver = Arc::new(DefaultAssetResolver::new(output_dir.clone()));
    let rewriter = Arc::new(DefaultContentRewriter);
    let detector = Arc::new(DefaultTechDetector);
    let scanner = Arc::new(DefaultSecretScanner::new());
    let api_discoverer = Arc::new(DefaultApiDiscoverer::new());
    let metrics = Arc::new(DefaultMetricsCollector::new());
    
    let mut monolith_storage_handle = None;
    let storage: Arc<dyn StorageProvider> = if args.warc {
        info!("Using WARC storage (ISO 28500)...");
        let warc_path = output_dir.join("archive.warc");
        Arc::new(WarcStorage::new(warc_path).await?)
    } else if args.monolith {
        info!("Using Monolith storage...");
        let monolith_path = output_dir.join("monolith.html");
        let ms = Arc::new(MonolithStorage::new(monolith_path));
        monolith_storage_handle = Some(ms.clone());
        ms
    } else {
        Arc::new(FileSystemStorage::new(output_dir))
    };

    // 5. Seed the queue
    let initial_asset = Asset::new(target_url.clone(), "".into(), AssetType::Html, 0);
    queue.push(initial_asset).await;

    // V2: Initialize TUI State
    let tui_state = Arc::new(tokio::sync::Mutex::new(cli::tui::TuiState::default()));
    let tui_dashboard = cli::tui::TuiDashboard::new(tui_state.clone());

    // 6. Initialize and run crawler
    let crawler = WebCrawler::new(
        client,
        queue,
        deduplicator,
        rate_limiter,
        parser,
        resolver,
        rewriter,
        detector,
        scanner,
        api_discoverer,
        metrics,
        storage,
        Some(db),
        Some(tui_state),
        args.depth,
        config.crawler.user_agent,
    );

    // Run Crawler and TUI concurrently
    let crawler_handle = tokio::spawn(async move {
        crawler.crawl().await
    });

    let tui_handle = tokio::spawn(async move {
        tui_dashboard.run().await
    });

    // Wait for either the crawler to finish or the user to exit via TUI
    tokio::select! {
        res = crawler_handle => {
            if let Ok(Ok(())) = res {
                info!("Crawler finished.");
            }
        }
        res = tui_handle => {
            if let Ok(Err(e)) = res {
                error!("TUI error: {}", e);
            }
        }
        _ = async {
            if let Some(handle) = browser_handle {
                let _ = handle.await;
            }
            futures::future::pending::<()>().await
        } => {}
    }

    if let Some(bc) = browser_client {
        bc.close().await;
    }

    // Finalize specialized storage if needed
    if let Some(ms) = monolith_storage_handle {
        info!("Finalizing monolith archive...");
        ms.finalize().await?;
    }

    info!("Archiving complete.");
    Ok(())
}
