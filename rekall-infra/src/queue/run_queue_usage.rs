use std::sync::Arc;

use rekall_core::claude::{SessionProvider, UsageReader};
use rekall_core::codex::CodexClient;
use rekall_core::Clock;

use super::usage_ceiling::{check_claude, check_codex, Verdict};

pub struct RunQueueUsage {
    pub claude: Arc<dyn UsageReader>,
    pub codex: Arc<dyn CodexClient>,
}

impl RunQueueUsage {
    pub(super) async fn check(
        &self,
        provider: SessionProvider,
        ceiling: Option<i32>,
        model: Option<&str>,
        refresh: bool,
        clock: &Clock,
    ) -> Verdict {
        let Some(ceiling) = ceiling else { return Verdict::go() };
        match provider {
            SessionProvider::Claude => {
                let reading = if refresh { self.claude.refresh().await } else { self.claude.current().await };
                check_claude(&reading, ceiling, model, clock())
            }
            SessionProvider::Codex => check_codex(self.codex.usage(refresh).await, ceiling, clock()),
        }
    }
}
