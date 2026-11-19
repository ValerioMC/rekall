use crate::connection::Mergeability;

/// The pull request (GitLab: merge request) a host holds for a branch: its number, the page that
/// shows it and whether it merges cleanly.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PullRequest {
    pub number: u64,
    pub url: String,
    pub mergeability: Mergeability,
}
