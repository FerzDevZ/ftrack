use crate::storage::Database;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;
use url::Url;

pub struct HashDeduplicator {
    visited_urls: Arc<Mutex<HashSet<Url>>>,
    #[allow(dead_code)]
    visited_hashes: Arc<Mutex<HashSet<String>>>,
    db: Option<Arc<Database>>,
}

impl HashDeduplicator {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            visited_urls: Arc::new(Mutex::new(HashSet::new())),
            visited_hashes: Arc::new(Mutex::new(HashSet::new())),
            db: None,
        }
    }

    pub fn with_db(db: Arc<Database>) -> Self {
        Self {
            visited_urls: Arc::new(Mutex::new(HashSet::new())),
            visited_hashes: Arc::new(Mutex::new(HashSet::new())),
            db: Some(db),
        }
    }

    pub async fn is_url_visited(&self, url: &Url) -> bool {
        {
            let urls = self.visited_urls.lock().await;
            if urls.contains(url) {
                return true;
            }
        }
        
        if let Some(db) = &self.db {
            if let Ok(true) = db.is_visited(url).await {
                // Sync back to memory
                let mut urls = self.visited_urls.lock().await;
                urls.insert(url.clone());
                return true;
            }
        }
        
        false
    }

    pub async fn mark_url_visited(&self, url: Url) {
        if let Some(db) = &self.db {
            let _ = db.mark_visited(&url).await;
        }
        let mut urls = self.visited_urls.lock().await;
        urls.insert(url);
    }
}
