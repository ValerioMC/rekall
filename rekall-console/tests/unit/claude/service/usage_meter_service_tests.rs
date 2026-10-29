use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;
use rekall_kernel::Instant;

use super::*;

/// Counts which reading was asked for.
#[derive(Default)]
struct FakeReader {
    cached: AtomicUsize,
    fresh: AtomicUsize,
}

#[async_trait]
impl UsageReader for FakeReader {
    async fn current(&self) -> ClaudeUsageView {
        self.cached.fetch_add(1, Ordering::SeqCst);
        ClaudeUsageView::ok(Vec::new(), Instant::now())
    }

    async fn refresh(&self) -> ClaudeUsageView {
        self.fresh.fetch_add(1, Ordering::SeqCst);
        ClaudeUsageView::unavailable()
    }
}

#[tokio::test]
async fn the_meter_reads_the_cache_unless_asked_to_check_again() {
    let reader = Arc::new(FakeReader::default());
    let meter = UsageMeterService::new(reader.clone());

    meter.reading(false).await;
    meter.reading(false).await;
    meter.reading(true).await;

    assert_eq!(reader.cached.load(Ordering::SeqCst), 2);
    assert_eq!(reader.fresh.load(Ordering::SeqCst), 1);
}
