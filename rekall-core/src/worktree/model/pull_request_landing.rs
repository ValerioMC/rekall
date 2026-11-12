/// What bringing an accepted task's pull request into the main branch came to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PullRequestLanding {
    /// The project does not run the pull request flow: the branch is merged locally.
    NotInFlow,
    Merged,
    /// An earlier acceptance already merged it.
    AlreadyMerged,
    /// The task never reached a claim, so no pull request exists.
    NothingToMerge,
    /// The host says the pull request conflicts with the main branch.
    Conflicting,
    /// The host or the task's state turned the merge down; the reason stays on the task.
    Refused(String),
}
