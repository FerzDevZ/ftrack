use crate::types::asset::Asset;

pub trait TechDetector: Send + Sync {
    fn detect(&self, asset: &Asset, content: &str) -> Vec<String>;
}

pub trait MetricsCollector: Send + Sync {
    fn record_request(&self, size: usize);
    fn report(&self) -> String;
}

pub trait SecretScanner: Send + Sync {
    fn scan(&self, url: &str, content: &str) -> Vec<crate::types::leak::Leak>;
}

pub trait ApiDiscovery: Send + Sync {
    fn discover(&self, url: &str, content: &str) -> Vec<crate::types::endpoint::Endpoint>;
}

pub mod tech_detect;
pub mod metrics;
pub mod secret_scanner;
pub mod api_discover;
pub mod signatures;
