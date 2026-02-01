use std::path::{Path, PathBuf};
use async_trait::async_trait;
use crate::types::error::Result;

use crate::storage::StorageProvider;

pub struct FileSystemStorage {
    base_dir: PathBuf,
}

impl FileSystemStorage {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }
}

#[async_trait]
impl StorageProvider for FileSystemStorage {
    async fn save(&self, path: &Path, content: &[u8]) -> Result<()> {
        let full_path = self.base_dir.join(path);
        if let Some(parent) = full_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(full_path, content).await?;
        Ok(())
    }

    async fn exists(&self, path: &Path) -> bool {
        self.base_dir.join(path).exists()
    }

    async fn load_metadata(&self, name: &str) -> Result<String> {
        let path = self.base_dir.join(format!("{}.json", name));
        Ok(tokio::fs::read_to_string(path).await?)
    }

    async fn save_metadata(&self, name: &str, content: &str) -> Result<()> {
        let path = self.base_dir.join(format!("{}.json", name));
        tokio::fs::write(path, content).await?;
        Ok(())
    }
}
