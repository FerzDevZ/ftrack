use crate::types::error::Result;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub crawler: CrawlerConfig,
    pub storage: StorageConfig,
    pub analyzer: AnalyzerConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CrawlerConfig {
    pub user_agent: String,
    pub max_depth: usize,
    pub concurrent_requests: usize,
    pub rate_limit_delay_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageConfig {
    pub output_dir: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyzerConfig {
    pub spa_mode: bool,
    pub detect_tech: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            crawler: CrawlerConfig {
                user_agent: "WebArchiverBot/1.0".to_string(),
                max_depth: 2,
                concurrent_requests: 5,
                rate_limit_delay_ms: 500,
            },
            storage: StorageConfig {
                output_dir: PathBuf::from("archive"),
            },
            analyzer: AnalyzerConfig {
                spa_mode: false,
                detect_tech: true,
            },
        }
    }
}

impl AppConfig {
    pub fn from_file(path: &std::path::Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&content).map_err(|e| crate::types::error::ArchivError::Config(e.to_string()))?;
        Ok(config)
    }
}
