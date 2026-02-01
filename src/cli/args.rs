use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Target URL to archive
    #[arg(required = true)]
    pub url: String,

    /// Output directory
    #[arg(short, long, default_value = "archive")]
    pub output: PathBuf,

    /// Maximum crawl depth
    #[arg(short, long, default_value_t = 2)]
    pub depth: usize,

    /// Enable SPA awareness (headless rendering)
    #[arg(long)]
    pub spa: bool,

    /// Enable passive API discovery
    #[arg(long)]
    pub analyze_api: bool,

    /// Output in WARC format (ISO 28500)
    #[arg(long)]
    pub warc: bool,

    /// Output as Monolith (all-in-one HTML, overrides standard storage)
    #[arg(long)]
    pub monolith: bool,

    /// Path to config file (TOML)
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    pub log_level: String,
}
