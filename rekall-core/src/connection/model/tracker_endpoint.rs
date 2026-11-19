use rekall_store::ConnectionKind;

use crate::connection::Secret;

/// Where a task manager's issues are written: the site (through its proxy, when it has one), the
/// place inside it (a Jira project key, a GitLab project path or a GitHub `owner/repo`) and the
/// stored credential that opens it.
#[derive(Debug)]
pub struct TrackerEndpoint {
    pub kind: ConnectionKind,
    pub site_url: String,
    pub proxy_url: Option<String>,
    pub location: String,
    pub credential: Secret,
}
