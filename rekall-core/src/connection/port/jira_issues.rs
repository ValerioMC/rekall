use async_trait::async_trait;

use crate::connection::{AtlassianCredential, AtlassianFailure, AtlassianSite, JiraIssue, JiraProject};

/// Reads issues and projects from a Jira site, so the console can import the issues as tasks and check a binding's project key.
#[async_trait]
pub trait JiraIssues: Send + Sync {
    /// The issues `jql` matches, in the order it asks for, at most `limit` of them.
    async fn search(&self, site: &AtlassianSite, credential: &AtlassianCredential, jql: &str, limit: u16) -> Result<Vec<JiraIssue>, AtlassianFailure>;

    /// The projects whose key or name holds `query` (every one when it is blank), at most `limit` of them.
    async fn projects(&self, site: &AtlassianSite, credential: &AtlassianCredential, query: &str, limit: u16) -> Result<Vec<JiraProject>, AtlassianFailure>;

    /// The project with exactly this key, or none when the site has no such project.
    async fn project(&self, site: &AtlassianSite, credential: &AtlassianCredential, key: &str) -> Result<Option<JiraProject>, AtlassianFailure>;
}
