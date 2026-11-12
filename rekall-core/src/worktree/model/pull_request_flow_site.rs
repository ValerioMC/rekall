/// Where a project's work happens: a folder to branch in, and a branch to open pull requests against.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PullRequestFlowSite {
    pub has_folder: bool,
    pub has_branch: bool,
}
