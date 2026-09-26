#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutoCommitStatus {
    /// The project does not auto-commit, or this claim is not one that commits.
    Off,
    /// The project auto-commits, but there was nothing in the working tree to commit.
    Skipped,
    /// A commit was made and logged against the step or task.
    Committed,
    /// The project auto-commits, but git refused; `detail` says why.
    Failed,
}
