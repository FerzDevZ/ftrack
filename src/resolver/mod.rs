use crate::types::error::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use url::Url;

pub trait AssetResolver: Send + Sync {
    fn resolve_local_path(&self, url: &Url) -> PathBuf;
}

pub trait ContentRewriter: Send + Sync {
    #[allow(dead_code)]
    fn rewrite(&self, content: &str, mapping: &HashMap<Url, PathBuf>) -> Result<String>;
}

pub mod asset;
pub mod rewrite;
