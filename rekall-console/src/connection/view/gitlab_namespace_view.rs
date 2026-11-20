use rekall_core::connection::{GitlabNamespace, GitlabNamespaceKind};

/// A group or a project of a GitLab instance, as a binding's target is chosen or confirmed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitlabNamespaceView {
    pub kind: GitlabNamespaceKind,
    pub path: String,
    pub name: String,
    pub visibility: Option<String>,
    pub default_branch: Option<String>,
    pub last_activity_at: Option<String>,
    pub subgroup_count: Option<u32>,
    pub project_count: Option<u32>,
}

impl From<GitlabNamespace> for GitlabNamespaceView {
    fn from(namespace: GitlabNamespace) -> Self {
        Self {
            kind: namespace.kind,
            path: namespace.path,
            name: namespace.name,
            visibility: namespace.visibility,
            default_branch: namespace.default_branch,
            last_activity_at: namespace.last_activity_at,
            subgroup_count: namespace.subgroup_count,
            project_count: namespace.project_count,
        }
    }
}
