use async_trait::async_trait;
use crate::storage::StorageProvider;
use crate::types::error::Result;
use std::path::{Path, PathBuf};
use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use chrono::Utc;

pub struct WarcStorage {
    #[allow(dead_code)]
    file_path: PathBuf,
    file: Arc<Mutex<File>>,
}

impl WarcStorage {
    pub async fn new(path: PathBuf) -> Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .await?;
            
        Ok(Self {
            file_path: path,
            file: Arc::new(Mutex::new(file)),
        })
    }

    async fn write_record(&self, headers: Vec<(String, String)>, body: &[u8]) -> Result<()> {
        let mut file = self.file.lock().await;
        let mut buffer = Vec::new();
        
        buffer.extend_from_slice(b"WARC/1.1\r\n");
        for (key, value) in headers {
            buffer.extend_from_slice(format!("{}: {}\r\n", key, value).as_bytes());
        }
        buffer.extend_from_slice(b"\r\n");
        buffer.extend_from_slice(body);
        buffer.extend_from_slice(b"\r\n\r\n");
        
        file.write_all(&buffer).await?;
        file.flush().await?;
        Ok(())
    }
}

#[async_trait]
impl StorageProvider for WarcStorage {
    async fn save(&self, path: &Path, content: &[u8]) -> Result<()> {
        let path_str = path.to_string_lossy();
        let now = Utc::now().to_rfc3339();
        let record_id = format!("<urn:uuid:{}>", Uuid::new_v4());
        
        let headers = vec![
            ("WARC-Type".to_string(), "resource".to_string()),
            ("WARC-Target-URI".to_string(), path_str.to_string()),
            ("WARC-Date".to_string(), now),
            ("WARC-Record-ID".to_string(), record_id),
            ("Content-Type".to_string(), "application/http; msgtype=response".to_string()),
            ("Content-Length".to_string(), content.len().to_string()),
        ];

        self.write_record(headers, content).await
    }

    async fn exists(&self, _path: &Path) -> bool {
        false
    }

    async fn load_metadata(&self, _name: &str) -> Result<String> {
        Ok("{}".to_string())
    }

    async fn save_metadata(&self, _name: &str, _content: &str) -> Result<()> {
        Ok(())
    }
}
