/// A binding's label, which a session names it by, and what the project uses the connection for:
/// the GitLab project path, the Kubernetes namespace or the Jira project key.
#[derive(Clone, Debug, Default)]
pub struct ProjectConnectionCommand {
    pub label: Option<String>,
    pub target: Option<String>,
}
