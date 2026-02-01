use async_trait::async_trait;
use crate::types::error::Result;
use std::path::Path;

#[async_trait]
pub trait StorageProvider: Send + Sync {
    async fn save(&self, path: &Path, content: &[u8]) -> Result<()>;
    #[allow(dead_code)]
    async fn exists(&self, path: &Path) -> bool;
    #[allow(dead_code)]
    async fn load_metadata(&self, name: &str) -> Result<String>;
    #[allow(dead_code)]
    async fn save_metadata(&self, name: &str, content: &str) -> Result<()>;
}
