use async_trait::async_trait;
use rekall_core::connection::{PullRequest, PullRequestHost, PullRequestMerge, TrackerEndpoint, TrackerFailure};
use rekall_store::ConnectionKind;

use super::{GithubPullRequestHost, GitlabPullRequestHost};

/// The pull request host of the connection's kind: GitHub or GitLab. Any other kind is a failure
/// the caller reports on the task.
pub struct ConnectionPullRequestHost {
    github: GithubPullRequestHost,
    gitlab: GitlabPullRequestHost,
}

impl ConnectionPullRequestHost {
    pub fn new() -> Self {
        Self { github: GithubPullRequestHost::new(), gitlab: GitlabPullRequestHost::new() }
    }

    fn host_of(&self, endpoint: &TrackerEndpoint) -> Result<&dyn PullRequestHost, TrackerFailure> {
        match endpoint.kind {
            ConnectionKind::Github => Ok(&self.github),
            ConnectionKind::Gitlab => Ok(&self.gitlab),
            ConnectionKind::Jira | ConnectionKind::Kubernetes | ConnectionKind::Grafana => {
                Err(TrackerFailure::Failed { reason: "only a GitHub or GitLab connection takes pull requests".into() })
            }
        }
    }
}

impl Default for ConnectionPullRequestHost {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PullRequestHost for ConnectionPullRequestHost {
    async fn open(&self, endpoint: &TrackerEndpoint, head: &str, base: &str, title: &str, body: &str) -> Result<PullRequest, TrackerFailure> {
        self.host_of(endpoint)?.open(endpoint, head, base, title, body).await
    }

    async fn find(&self, endpoint: &TrackerEndpoint, head: &str) -> Result<Option<PullRequest>, TrackerFailure> {
        self.host_of(endpoint)?.find(endpoint, head).await
    }

    async fn merge(&self, endpoint: &TrackerEndpoint, pull_request: &PullRequest) -> Result<PullRequestMerge, TrackerFailure> {
        self.host_of(endpoint)?.merge(endpoint, pull_request).await
    }

    async fn close_issue(&self, endpoint: &TrackerEndpoint, issue_key: &str) -> Result<(), TrackerFailure> {
        self.host_of(endpoint)?.close_issue(endpoint, issue_key).await
    }
}

#[cfg(test)]
#[path = "../../tests/unit/tracker/connection_pull_request_host_tests.rs"]
mod tests;
