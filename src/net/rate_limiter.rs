use std::sync::Arc;
use tokio::sync::Semaphore;
use std::collections::HashMap;
use tokio::sync::Mutex;
use std::time::Duration;
use tokio::time::sleep;

pub struct RateLimiter {
    semaphores: Mutex<HashMap<String, Arc<Semaphore>>>,
    max_concurrent: usize,
    delay_ms: u64,
}

impl RateLimiter {
    pub fn new(max_concurrent: usize, delay_ms: u64) -> Self {
        Self {
            semaphores: Mutex::new(HashMap::new()),
            max_concurrent,
            delay_ms,
        }
    }

    #[allow(dead_code)]
    pub fn get_host(url: &url::Url) -> String {
        url.host_str().unwrap_or("unknown").to_string()
    }

    pub async fn acquire(&self, host: &str) -> RateLimitGuard {
        let mut semaphores = self.semaphores.lock().await;
        let sem = semaphores
            .entry(host.to_string())
            .or_insert_with(|| Arc::new(Semaphore::new(self.max_concurrent)))
            .clone();
        
        let permit = sem.acquire_owned().await.expect("Semaphore closed");
        
        if self.delay_ms > 0 {
            sleep(Duration::from_millis(self.delay_ms)).await;
        }

        RateLimitGuard { _permit: permit }
    }
}

pub struct RateLimitGuard {
    _permit: tokio::sync::OwnedSemaphorePermit,
}
