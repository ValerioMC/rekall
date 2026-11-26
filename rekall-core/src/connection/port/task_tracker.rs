use async_trait::async_trait;

use crate::connection::{TrackerEndpoint, TrackerFailure, TrackerIssue, TrackerLink};
use crate::picture::PicturePath;

/// Writes a task to the GitLab, GitHub or Jira project the task manager names. The endpoint's kind picks the API.
#[async_trait]
pub trait TaskTracker: Send + Sync {
    async fn create(&self, endpoint: &TrackerEndpoint, issue: &TrackerIssue) -> Result<TrackerLink, TrackerFailure>;

    /// Removes the issue `key` names. A tracker that cannot delete issues closes it instead.
    async fn delete(&self, endpoint: &TrackerEndpoint, key: &str) -> Result<(), TrackerFailure>;

    /// Rewrites the title and the description of the issue `key` names.
    async fn update(&self, endpoint: &TrackerEndpoint, key: &str, issue: &TrackerIssue) -> Result<(), TrackerFailure>;

    /// Uploads a picture to the repository, on Jira to the issue `issue_key` names, and answers how
    /// the tracker refers to its copy: an address, on Jira Data Center the attachment's name.
    async fn upload_picture(&self, endpoint: &TrackerEndpoint, issue_key: Option<&str>, picture: &PicturePath, bytes: Vec<u8>) -> Result<String, TrackerFailure>;
}
