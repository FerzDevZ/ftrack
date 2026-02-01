pub mod crawler;
pub mod scheduler;
pub mod queue;
pub mod deduplicator;

use crate::types::asset::Asset;
use crate::types::error::Result;
use async_trait::async_trait;

#[async_trait]
#[allow(dead_code)]
pub trait Crawler: Send + Sync {
    async fn run(&self) -> Result<()>;
}

#[allow(dead_code)]
pub trait Scheduler: Send + Sync {
    fn push(&self, asset: Asset);
    fn next(&self) -> Option<Asset>;
}

#[allow(dead_code)]
pub trait Deduplicator: Send + Sync {
    fn is_duplicate(&self, asset: &Asset) -> bool;
    fn mark_visited(&self, asset: &Asset);
}
