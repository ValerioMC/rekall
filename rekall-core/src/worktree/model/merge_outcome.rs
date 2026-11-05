/// What merging a task's branch into the project's current `HEAD` came to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MergeOutcome {
    Merged,
    /// git refused or hit a conflict; the merge was abandoned and the project is as it was.
    Refused(String),
}
