use async_trait::async_trait;

use crate::connection::{PullRequest, PullRequestMerge, TrackerEndpoint, TrackerFailure};

/// Opens and merges the pull request of a task's branch on the GitHub or GitLab project the task
/// manager names, and closes its issue. A refusal the host explains with a verdict is a value; an
/// `Err` is a rejected credential, the network or a 5xx, and the caller tries again later.
#[async_trait]
pub trait PullRequestHost: Send + Sync {
    /// Opens a pull request from `head` into `base`, or returns the one already open for `head`.
    async fn open(&self, endpoint: &TrackerEndpoint, head: &str, base: &str, title: &str, body: &str) -> Result<PullRequest, TrackerFailure>;

    async fn find(&self, endpoint: &TrackerEndpoint, head: &str) -> Result<Option<PullRequest>, TrackerFailure>;

    /// Merges with a merge commit, never a squash or a rebase.
    async fn merge(&self, endpoint: &TrackerEndpoint, pull_request: &PullRequest) -> Result<PullRequestMerge, TrackerFailure>;

    async fn close_issue(&self, endpoint: &TrackerEndpoint, issue_key: &str) -> Result<(), TrackerFailure>;
}
