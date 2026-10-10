use rekall_common::Id;
use rekall_model::token_usage;
use serde::Serialize;

/// Every session's tokens on one step, or on its task outside any step, summed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenTallyView {
    pub task_id: Id,
    pub step_id: Option<Id>,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_creation_tokens: i64,
    pub cache_read_tokens: i64,
}

impl TokenTallyView {
    /// One tally per task and step the rows touch, in the order each first appears.
    pub fn summed(rows: &[token_usage::Model]) -> Vec<Self> {
        let mut tallies: Vec<Self> = Vec::new();
        for row in rows {
            let at = match tallies.iter().position(|t| t.task_id == row.task_id && t.step_id == row.step_id) {
                Some(at) => at,
                None => {
                    tallies.push(Self::empty(row.task_id, row.step_id));
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

    fn empty(task_id: Id, step_id: Option<Id>) -> Self {
        Self { task_id, step_id, input_tokens: 0, output_tokens: 0, cache_creation_tokens: 0, cache_read_tokens: 0 }
    }
}
