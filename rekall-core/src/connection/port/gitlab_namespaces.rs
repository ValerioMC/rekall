use async_trait::async_trait;

use crate::connection::{GitlabNamespace, TrackerEndpoint, TrackerFailure};

/// Reads the groups and projects of a GitLab instance, so a binding's target can be checked
/// against it and chosen from what the credential can read. The endpoint's location is the path
/// asked about, and its kind is GitLab.
#[async_trait]
pub trait GitlabNamespaces: Send + Sync {
    /// The project or the group at the endpoint's location, a project first; none when GitLab has neither.
    async fn lookup(&self, endpoint: &TrackerEndpoint) -> Result<Option<GitlabNamespace>, TrackerFailure>;

    /// The subgroups and projects directly under the group at the endpoint's location (the
    /// top-level groups when it is blank) whose path holds `query`, groups first, at most `limit`.
    async fn children(&self, endpoint: &TrackerEndpoint, query: &str, limit: u16) -> Result<Vec<GitlabNamespace>, TrackerFailure>;
}
