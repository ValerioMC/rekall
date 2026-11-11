//! What a project's task manager needs of its binding: a GitLab, GitHub or Jira one, whose target
//! says where the issues go.

use rekall_kernel::{jstr, RekallError, Result};
use rekall_store::ConnectionKind;

use crate::connection::rule::jira_rules::project_key_of;

/// Where the issues go: a Jira project key, a GitLab project path or a GitHub `owner/repo`.
pub(in crate::connection) fn tracker_location_of(kind: ConnectionKind, target: Option<&str>) -> Result<String> {
    match kind {
        ConnectionKind::Jira => project_key_of(target),
        ConnectionKind::Gitlab => path_of(target, "GitLab", "group/project"),
        ConnectionKind::Github => path_of(target, "GitHub", "owner/repo").and_then(require_two_segments),
        ConnectionKind::Kubernetes | ConnectionKind::Grafana => {
            Err(RekallError::illegal("Only a GitLab, GitHub or Jira connection can be a project's task manager."))
        }
    }
}

fn path_of(target: Option<&str>, name: &str, shape: &str) -> Result<String> {
    let path = target.map(jstr::strip).unwrap_or_default().trim_matches('/');
    let segments: Vec<&str> = path.split('/').collect();
    if segments.len() < 2 || segments.iter().any(|segment| segment.is_empty() || segment.contains(char::is_whitespace)) {
        return Err(RekallError::illegal(format!("Set this project's target for the {name} connection to the repository's path, such as {shape}.")));
    }
    Ok(path.to_string())
}

fn require_two_segments(path: String) -> Result<String> {
    if path.split('/').count() != 2 {
        return Err(RekallError::illegal("Set this project's target for the GitHub connection to owner/repo."));
    }
    Ok(path)
}

#[cfg(test)]
#[path = "../../../tests/unit/connection/rule/tracker_rules_tests.rs"]
mod tests;
