use rekall_kernel::Id;
use rekall_store::{token_usage, SessionProvider};

use crate::token::session_provider;
use serde::Serialize;

/// A provider's tokens on one step or directly on a task.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenTallyView {
    pub task_id: Id,
    pub step_id: Option<Id>,
    pub provider: SessionProvider,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_creation_tokens: i64,
    pub cache_read_tokens: i64,
}

impl TokenTallyView {
    /// One tally per task, optional step, and provider.
    pub fn summed(rows: &[token_usage::Model]) -> Vec<Self> {
        let mut tallies: Vec<Self> = Vec::new();
        for row in rows {
            let provider = session_provider(&row.session_id);
            let at = match tallies.iter().position(|t| t.task_id == row.task_id && t.step_id == row.step_id && t.provider == provider) {
                Some(at) => at,
                None => {
                    tallies.push(Self::empty(row.task_id, row.step_id, provider));
                    tallies.len() - 1
                }
            };
            let tally = &mut tallies[at];
            tally.input_tokens += row.input_tokens;
            tally.output_tokens += row.output_tokens;
            tally.cache_creation_tokens += row.cache_creation_tokens;
            tally.cache_read_tokens += row.cache_read_tokens;
        }
        tallies
    }

    fn empty(task_id: Id, step_id: Option<Id>, provider: SessionProvider) -> Self {
        Self { task_id, step_id, provider, input_tokens: 0, output_tokens: 0, cache_creation_tokens: 0, cache_read_tokens: 0 }
    }
}
