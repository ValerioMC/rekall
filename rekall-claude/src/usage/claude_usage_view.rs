use rekall_common::Instant;
use serde::Serialize;

use super::{Limit, UsageStatus};

/// What the console needs to draw the meter. `retry_at` is set whenever a wait is in force,
/// whatever the status: a last good reading served during one carries it too.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeUsageView {
    pub status: UsageStatus,
    pub limits: Vec<Limit>,
    pub fetched_at: Instant,
    pub retry_at: Option<Instant>,
}

impl ClaudeUsageView {
    pub fn ok(limits: Vec<Limit>, fetched_at: Instant) -> Self {
        Self { status: UsageStatus::Ok, limits, fetched_at, retry_at: None }
    }

    pub fn unauthenticated() -> Self {
        Self { status: UsageStatus::Unauthenticated, limits: Vec::new(), fetched_at: Instant::now(), retry_at: None }
    }

    pub fn rate_limited(retry_at: Instant) -> Self {
        Self { status: UsageStatus::RateLimited, limits: Vec::new(), fetched_at: Instant::now(), retry_at: Some(retry_at) }
    }

    pub fn unavailable() -> Self {
        Self { status: UsageStatus::Unavailable, limits: Vec::new(), fetched_at: Instant::now(), retry_at: None }
    }

    /// The same reading, marked as one that cannot be refreshed before `retry_at`.
    pub fn held_until(&self, retry_at: Instant) -> Self {
        Self { retry_at: Some(retry_at), ..self.clone() }
    }
}
