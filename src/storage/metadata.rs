use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct CrawlMetadata {
    pub target_url: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub tech_stack: Vec<String>,
    pub asset_mapping: HashMap<String, String>, // URL -> LocalPath
    pub stats: HashMap<String, String>,
}

impl CrawlMetadata {
    #[allow(dead_code)]
    pub fn new(target_url: String) -> Self {
        Self {
            target_url,
            timestamp: chrono::Utc::now(),
            tech_stack: Vec::new(),
            asset_mapping: HashMap::new(),
            stats: HashMap::new(),
        }
    }
}
