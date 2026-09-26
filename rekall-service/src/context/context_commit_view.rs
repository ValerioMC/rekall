use rekall_model::{commit_reference, task_step};

/// One logged commit a task hands to `rekall_context`: the hash, the subject it was committed
/// with, the step it was logged against, and the diff it introduced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContextCommitView {
    pub commit_hash: String,
    pub comment: String,
    pub step_title: Option<String>,
    pub diff: Option<String>,
}

impl ContextCommitView {
    pub fn of(reference: &commit_reference::Model, step: Option<&task_step::Model>) -> Self {
        Self {
            commit_hash: reference.commit_hash.clone(),
            comment: reference.comment.clone(),
            step_title: step.map(|s| s.title.clone()),
            diff: reference.diff.clone(),
        }
    }
}
