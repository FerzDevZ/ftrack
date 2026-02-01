use crate::types::asset::Asset;
use crate::storage::Database;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct PriorityQueue {
    queue: Arc<Mutex<VecDeque<Asset>>>,
    db: Option<Arc<Database>>,
}

impl PriorityQueue {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            db: None,
        }
    }

    pub fn with_db(db: Arc<Database>) -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            db: Some(db),
        }
    }

    pub async fn push(&self, asset: Asset) {
        if let Some(db) = &self.db {
            let _ = db.push_queue(asset.clone()).await;
        }
        let mut q = self.queue.lock().await;
        q.push_back(asset);
    }

    pub async fn pop(&self) -> Option<Asset> {
        let mut q = self.queue.lock().await;
        if let Some(asset) = q.pop_front() {
            if let Some(_db) = &self.db {
                // In a more robust system, we'd mark it as "processing" in DB
                // For now, pop_queue in DB deletes it, but here we just pop from memory.
                // We need to ensure DB and memory are synced.
                // Actually, if we use DB as source of truth, memory is just a cache.
            }
            Some(asset)
        } else if let Some(db) = &self.db {
            // Try to load from DB if memory is empty
            if let Ok(Some(asset)) = db.pop_queue().await {
                Some(asset)
            } else {
                None
            }
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub async fn is_empty(&self) -> bool {
        self.len().await == 0
    }

    pub async fn len(&self) -> usize {
        self.queue.lock().await.len()
    }
}
