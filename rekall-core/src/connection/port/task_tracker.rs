use async_trait::async_trait;

use crate::connection::{TrackerEndpoint, TrackerFailure, TrackerIssue, TrackerLink};

/// Writes a task to the GitLab, GitHub or Jira project the task manager names. The endpoint's kind picks the API.
#[async_trait]
pub trait TaskTracker: Send + Sync {
    async fn create(&self, endpoint: &TrackerEndpoint, issue: &TrackerIssue) -> Result<TrackerLink, TrackerFailure>;

    /// Rewrites the title and the description of the issue `key` names.
    async fn update(&self, endpoint: &TrackerEndpoint, key: &str, issue: &TrackerIssue) -> Result<(), TrackerFailure>;
}
