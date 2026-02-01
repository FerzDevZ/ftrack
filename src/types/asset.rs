use std::path::PathBuf;
use url::Url;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssetType {
    Html,
    Css,
    Js,
    Image,
    Font,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub url: Url,
    pub local_path: PathBuf,
    pub asset_type: AssetType,
    pub mime_type: Option<String>,
    pub hash: Option<String>,
    pub depth: usize,
    pub priority: i32,
}

impl Asset {
    pub fn new(url: Url, local_path: PathBuf, asset_type: AssetType, depth: usize) -> Self {
        Self {
            url,
            local_path,
            asset_type,
            mime_type: None,
            hash: None,
            depth,
            priority: 0,
        }
    }
}
