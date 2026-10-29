/// A commit read with its patch, ready to be logged against a task.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoggedCommit {
    pub hash: String,
    pub subject: String,
    pub diff: Option<String>,
}
