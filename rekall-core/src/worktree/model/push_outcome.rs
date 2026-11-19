/// What pushing a task's branch came to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PushOutcome {
    Pushed,
    /// The remote already holds every commit of the branch.
    UpToDate,
    /// The remote or git said no (no remote, no permission, a branch that moved on): the first lines of its message.
    Refused(String),
}
