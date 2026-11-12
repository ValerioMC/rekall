/// What merging a task's branch into the project's current `HEAD` came to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MergeOutcome {
    Merged,
    /// git stopped on conflicting files; the merge was abandoned and the project is as it was.
    Conflicts(String),
    /// git refused the merge for another reason; it was abandoned and the project is as it was.
    Refused(String),
}
