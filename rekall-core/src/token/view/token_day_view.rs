use std::collections::HashMap;

use rekall_kernel::{Id, Instant};
use rekall_store::{token_usage, SessionProvider};
use serde::Serialize;

use crate::token::session_provider;

/// A provider's tokens on one task on one day, summed over its steps.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenDayView {
    pub task_id: Id,
    /// Midnight UTC of the day.
    pub day: Instant,
    pub provider: SessionProvider,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_creation_tokens: i64,
    pub cache_read_tokens: i64,
}

impl TokenDayView {
    /// One entry per task, day and provider, oldest day first.
    pub fn summed(rows: &[token_usage::Model]) -> Vec<Self> {
        let mut days = HashMap::new();
        for row in rows {
            let provider = session_provider(&row.session_id);
            let day = days.entry((row.spent_on, row.task_id, provider))
                .or_insert_with(|| Self::empty(row.task_id, row.spent_on, provider));
            day.input_tokens += row.input_tokens;
            day.output_tokens += row.output_tokens;
            day.cache_creation_tokens += row.cache_creation_tokens;
            day.cache_read_tokens += row.cache_read_tokens;
        }
        let mut days: Vec<Self> = days.into_values().collect();
        days.sort_by_key(|d| (d.day, d.task_id, d.provider.name()));
        days
    }

    fn empty(task_id: Id, day: Instant, provider: SessionProvider) -> Self {
        Self { task_id, day, provider, input_tokens: 0, output_tokens: 0, cache_creation_tokens: 0, cache_read_tokens: 0 }
    }
}
