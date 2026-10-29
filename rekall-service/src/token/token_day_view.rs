use rekall_common::{Id, Instant};
use rekall_store::token_usage;
use serde::Serialize;

/// Every session's tokens on one task on one day, summed over its steps.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenDayView {
    pub task_id: Id,
    /// Midnight UTC of the day.
    pub day: Instant,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_creation_tokens: i64,
    pub cache_read_tokens: i64,
}

impl TokenDayView {
    /// One entry per task and day the rows touch, oldest day first.
    pub fn summed(rows: &[token_usage::Model]) -> Vec<Self> {
        let mut days: Vec<Self> = Vec::new();
        for row in rows {
            let at = match days.iter().position(|d| d.task_id == row.task_id && d.day == row.spent_on) {
                Some(at) => at,
                None => {
                    days.push(Self::empty(row.task_id, row.spent_on));
                    days.len() - 1
                }
            };
            let day = &mut days[at];
            day.input_tokens += row.input_tokens;
            day.output_tokens += row.output_tokens;
            day.cache_creation_tokens += row.cache_creation_tokens;
            day.cache_read_tokens += row.cache_read_tokens;
        }
        days.sort_by_key(|d| (d.day, d.task_id));
        days
    }

    fn empty(task_id: Id, day: Instant) -> Self {
        Self { task_id, day, input_tokens: 0, output_tokens: 0, cache_creation_tokens: 0, cache_read_tokens: 0 }
    }
}
