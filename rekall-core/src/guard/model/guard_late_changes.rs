use serde::{Deserialize, Serialize};

/// The files a task's worktree still holds uncommitted once all its work is claimed and not yet
/// accepted. `step_to_claim` is the last claimed step, none on a task with no steps.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuardLateChanges {
    pub task_anchor: String,
    pub files: Vec<String>,
    pub step_to_claim: Option<u32>,
}
