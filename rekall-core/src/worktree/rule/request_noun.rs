use rekall_store::ConnectionKind;

/// What the host calls the request that brings a branch in: GitLab says merge request, any other host pull request.
pub fn request_noun(kind: ConnectionKind) -> &'static str {
    match kind {
        ConnectionKind::Gitlab => "merge request",
        _ => "pull request",
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/worktree/rule/request_noun_tests.rs"]
mod tests;
