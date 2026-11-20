use async_trait::async_trait;

use crate::connection::{GitlabNamespace, GitlabNamespaceKind, GitlabNamespaces, TrackerEndpoint, TrackerFailure};

/// A GitLab instance a test describes: the groups and projects it has, or the failure every call answers with.
#[derive(Default)]
pub struct ScriptedGitlabNamespaces {
    namespaces: Vec<GitlabNamespace>,
    failure: Option<TrackerFailure>,
}

impl ScriptedGitlabNamespaces {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn failing(failure: TrackerFailure) -> Self {
        Self { namespaces: Vec::new(), failure: Some(failure) }
    }

    pub fn with_group(self, path: &str) -> Self {
        self.with(GitlabNamespaceKind::Group, path)
    }

    pub fn with_project(self, path: &str) -> Self {
        self.with(GitlabNamespaceKind::Project, path)
    }

    fn with(mut self, kind: GitlabNamespaceKind, path: &str) -> Self {
        let name = path.rsplit('/').next().unwrap_or(path).to_string();
        self.namespaces.push(GitlabNamespace { kind, path: path.into(), name, visibility: Some("private".into()), default_branch: Some("main".into()), last_activity_at: None, subgroup_count: None, project_count: None });
        self
    }

    fn failed(&self) -> Result<(), TrackerFailure> {
        self.failure.clone().map_or(Ok(()), Err)
    }
}

#[async_trait]
impl GitlabNamespaces for ScriptedGitlabNamespaces {
    async fn lookup(&self, endpoint: &TrackerEndpoint) -> Result<Option<GitlabNamespace>, TrackerFailure> {
        self.failed()?;
        Ok(self.namespaces.iter().find(|namespace| namespace.path.eq_ignore_ascii_case(&endpoint.location)).cloned())
    }

    async fn children(&self, endpoint: &TrackerEndpoint, query: &str, limit: u16) -> Result<Vec<GitlabNamespace>, TrackerFailure> {
        self.failed()?;
        let parent = endpoint.location.trim_matches('/');
        let directly_under = |path: &str| path.rsplit_once('/').map_or("", |(group, _)| group).eq_ignore_ascii_case(parent);
        let listed = self.namespaces.iter().filter(|namespace| directly_under(&namespace.path) && namespace.path.contains(query));
        Ok(listed.take(usize::from(limit)).cloned().collect())
    }
}
