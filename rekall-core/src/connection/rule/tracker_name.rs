use rekall_store::ConnectionKind;

/// What a message to the person calls the system a task manager is.
pub fn tracker_name(kind: ConnectionKind) -> &'static str {
    match kind {
        ConnectionKind::Jira => "Jira",
        ConnectionKind::Gitlab => "GitLab",
        ConnectionKind::Github => "GitHub",
        ConnectionKind::Kubernetes | ConnectionKind::Grafana | ConnectionKind::Confluence => "The task manager",
    }
}
