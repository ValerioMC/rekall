/// Whether a path on GitLab names a group (or subgroup) or a project.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GitlabNamespaceKind {
    Group,
    Project,
}

/// A group or a project GitLab has at a path. The counts are filled only for a group asked by its
/// path, and are none when GitLab does not total them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitlabNamespace {
    pub kind: GitlabNamespaceKind,
    pub path: String,
    pub name: String,
    pub visibility: Option<String>,
    pub default_branch: Option<String>,
    pub last_activity_at: Option<String>,
    pub subgroup_count: Option<u32>,
    pub project_count: Option<u32>,
}
