use crate::analyzer::MetricsCollector;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

pub struct DefaultMetricsCollector {
    request_count: AtomicUsize,
    total_bytes: AtomicUsize,
    start_time: Instant,
}

impl DefaultMetricsCollector {
    pub fn new() -> Self {
        Self {
            request_count: AtomicUsize::new(0),
            total_bytes: AtomicUsize::new(0),
            start_time: Instant::now(),
        }
    }
}

impl MetricsCollector for DefaultMetricsCollector {
    fn record_request(&self, size: usize) {
        self.request_count.fetch_add(1, Ordering::Relaxed);
        self.total_bytes.fetch_add(size, Ordering::Relaxed);
    }

    fn report(&self) -> String {
        let duration = self.start_time.elapsed();
        let count = self.request_count.load(Ordering::Relaxed);
        let bytes = self.total_bytes.load(Ordering::Relaxed);
        
        format!(
            "Crawl Summary:\n - Requests: {}\n - Total Data: {:.2} MB\n - Duration: {:.2}s",
            count,
            bytes as f64 / 1_048_576.0,
            duration.as_secs_f64()
        )
    }
}
