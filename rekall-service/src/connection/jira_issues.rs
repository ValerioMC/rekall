use async_trait::async_trait;

use super::{JiraCredential, JiraFailure, JiraIssue};

/// Reads issues from a Jira site, so the console can import them as tasks.
#[async_trait]
pub trait JiraIssues: Send + Sync {
    /// The issues `jql` matches, in the order it asks for, at most `limit` of them.
    async fn search(&self, site: &str, credential: &JiraCredential, jql: &str, limit: u16) -> Result<Vec<JiraIssue>, JiraFailure>;
}
