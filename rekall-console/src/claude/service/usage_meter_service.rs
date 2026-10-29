use std::sync::Arc;

use rekall_core::claude::{ClaudeUsageView, UsageReader};

/// The meter's reading: the cached one, or a new one when the person asks to check again.
#[derive(Clone)]
pub struct UsageMeterService {
    reader: Arc<dyn UsageReader>,
}

impl UsageMeterService {
    pub fn new(reader: Arc<dyn UsageReader>) -> Self {
        Self { reader }
    }

    pub async fn reading(&self, refresh: bool) -> ClaudeUsageView {
        if refresh {
            return self.reader.refresh().await;
        }
        self.reader.current().await
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/claude/service/usage_meter_service_tests.rs"]
mod tests;
