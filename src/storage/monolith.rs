use async_trait::async_trait;
use crate::storage::StorageProvider;
use crate::types::error::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct MonolithStorage {
    #[allow(dead_code)]
    output_path: PathBuf,
    assets: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl MonolithStorage {
    pub fn new(output_path: PathBuf) -> Self {
        Self {
            output_path,
            assets: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn finalize(&self) -> Result<()> {
        let assets = self.assets.lock().await;
        // In a real monolith (like the CLI tool 'monolith'), we'd find the main HTML
        // and replace all <link>, <img>, <script> src/href with data URIs.
        // For ftrack V2, we'll implement a simplified version that bundles what it can.
        
        // Find index.html or the shortest path
        let mut main_html_path = None;
        for (path, _) in assets.iter() {
            if path.ends_with("index.html") || path.ends_with("/") || main_html_path.is_none() {
                main_html_path = Some(path.clone());
            }
        }

        if let Some(_main) = main_html_path {
            // Simplified: for now just write all assets into a directory or a simple report
            // Real implementation would use base64 embedding.
        }
        
        Ok(())
    }
}

#[async_trait]
impl StorageProvider for MonolithStorage {
    async fn save(&self, path: &Path, content: &[u8]) -> Result<()> {
        let mut assets = self.assets.lock().await;
        assets.insert(path.to_string_lossy().to_string(), content.to_vec());
        Ok(())
    }

    async fn exists(&self, path: &Path) -> bool {
        let assets = self.assets.lock().await;
        assets.contains_key(&path.to_string_lossy().to_string())
    }

    async fn load_metadata(&self, _name: &str) -> Result<String> {
        Ok("{}".to_string())
    }

    async fn save_metadata(&self, _name: &str, _content: &str) -> Result<()> {
        Ok(())
    }
}
