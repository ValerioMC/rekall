/// A commit a step logged on a task's branch: the hash, its subject and the step it was logged for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StepCommit {
    pub step_title: Option<String>,
    pub hash: String,
    pub subject: String,
}
