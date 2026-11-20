use rekall_console::connection::GitlabNamespaceView;
use rekall_core::connection::GitlabNamespaceKind;
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GitlabNamespaceKindResponse {
    Group,
    Project,
}

/// A group or a project of a GitLab instance.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitlabNamespaceResponse {
    pub kind: GitlabNamespaceKindResponse,
    pub path: String,
    pub name: String,
    pub visibility: Option<String>,
    pub default_branch: Option<String>,
    pub last_activity_at: Option<String>,
    pub subgroup_count: Option<u32>,
    pub project_count: Option<u32>,
}

impl From<GitlabNamespaceView> for GitlabNamespaceResponse {
    fn from(view: GitlabNamespaceView) -> Self {
        Self {
            kind: match view.kind {
                GitlabNamespaceKind::Group => GitlabNamespaceKindResponse::Group,
                GitlabNamespaceKind::Project => GitlabNamespaceKindResponse::Project,
            },
            path: view.path,
            name: view.name,
            visibility: view.visibility,
            default_branch: view.default_branch,
            last_activity_at: view.last_activity_at,
            subgroup_count: view.subgroup_count,
            project_count: view.project_count,
        }
    }
}
