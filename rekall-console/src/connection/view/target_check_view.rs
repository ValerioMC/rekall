use crate::connection::view::gitlab_namespace_view::GitlabNamespaceView;

/// What asking GitLab about a typed path came to. Every outcome is a value: the console shows each
/// as its own answer and only a found path or a blank one can be bound.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TargetCheckView {
    Found(GitlabNamespaceView),
    NotFound { nearest: Option<String> },
    OutsideRoot { root: String },
    NoCredential,
    Unreachable { reason: String },
}
