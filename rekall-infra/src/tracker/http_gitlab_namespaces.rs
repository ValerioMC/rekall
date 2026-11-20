use std::time::Duration;

use async_trait::async_trait;
use reqwest::header::{ACCEPT, USER_AGENT};
use reqwest::{Response, StatusCode};
use rekall_core::connection::{GitlabNamespace, GitlabNamespaceKind, GitlabNamespaces, TrackerEndpoint, TrackerFailure};
use serde::de::DeserializeOwned;
use serde::Deserialize;

use super::http_task_tracker::{checked, gitlab_project, signature_of};
use crate::probes::{client_through, failure_chain};

const TIMEOUT: Duration = Duration::from_secs(10);
const TOTAL_HEADER: &str = "x-total";

#[derive(Deserialize)]
struct GitlabGroup {
    full_path: String,
    #[serde(default)]
    name: String,
    visibility: Option<String>,
}

#[derive(Deserialize)]
struct GitlabProject {
    path_with_namespace: String,
    #[serde(default)]
    name: String,
    visibility: Option<String>,
    default_branch: Option<String>,
    last_activity_at: Option<String>,
}

impl From<GitlabGroup> for GitlabNamespace {
    fn from(group: GitlabGroup) -> Self {
        Self {
            kind: GitlabNamespaceKind::Group,
            path: group.full_path,
            name: group.name,
            visibility: group.visibility,
            default_branch: None,
            last_activity_at: None,
            subgroup_count: None,
            project_count: None,
        }
    }
}

impl From<GitlabProject> for GitlabNamespace {
    fn from(project: GitlabProject) -> Self {
        Self {
            kind: GitlabNamespaceKind::Project,
            path: project.path_with_namespace,
            name: project.name,
            visibility: project.visibility,
            default_branch: project.default_branch,
            last_activity_at: project.last_activity_at,
            subgroup_count: None,
            project_count: None,
        }
    }
}

/// GitLab's group and project API over HTTPS with `reqwest`, through the connection's proxy when
/// it names one. A path GitLab answers 404 to is absent, which is also what a path the credential
/// cannot read looks like.
pub struct HttpGitlabNamespaces {
    timeout: Duration,
}

impl HttpGitlabNamespaces {
    pub fn new() -> Self {
        Self { timeout: TIMEOUT }
    }

    async fn send(&self, endpoint: &TrackerEndpoint, path: &str, query: &[(&str, String)]) -> Result<Option<Response>, TrackerFailure> {
        let client = client_through(endpoint.proxy_url.as_deref(), self.timeout).map_err(|reason| TrackerFailure::Failed { reason })?;
        let (name, value) = signature_of(endpoint)?;
        let site = endpoint.site_url.trim_end_matches('/');
        let response = client
            .get(format!("{site}/api/v4{path}"))
            .header(name, value)
            .header(USER_AGENT, "rekall")
            .header(ACCEPT, "application/json")
            .query(query)
            .send()
            .await
            .map_err(|failure| TrackerFailure::Failed { reason: failure_chain(&failure) })?;
        if response.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        checked(response).await.map(Some)
    }

    async fn read<T: DeserializeOwned>(&self, endpoint: &TrackerEndpoint, path: &str, query: &[(&str, String)]) -> Result<Option<T>, TrackerFailure> {
        let Some(response) = self.send(endpoint, path, query).await? else { return Ok(None) };
        response.json::<T>().await.map(Some).map_err(|failure| TrackerFailure::Failed { reason: format!("its answer did not read as expected ({failure})") })
    }

    async fn total(&self, endpoint: &TrackerEndpoint, path: &str, query: &[(&str, String)]) -> Result<Option<u32>, TrackerFailure> {
        let Some(response) = self.send(endpoint, path, query).await? else { return Ok(None) };
        Ok(response.headers().get(TOTAL_HEADER).and_then(|total| total.to_str().ok()).and_then(|total| total.parse().ok()))
    }

    async fn group_with_totals(&self, endpoint: &TrackerEndpoint, group: GitlabGroup) -> Result<GitlabNamespace, TrackerFailure> {
        let encoded = gitlab_project(&group.full_path);
        let one = ("per_page", "1".to_string());
        let subgroups = self.total(endpoint, &format!("/groups/{encoded}/subgroups"), &[one.clone()]).await?;
        let projects = self.total(endpoint, &format!("/groups/{encoded}/projects"), &[one, ("include_subgroups", "true".to_string())]).await?;
        Ok(GitlabNamespace { subgroup_count: subgroups, project_count: projects, ..group.into() })
    }
}

impl Default for HttpGitlabNamespaces {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GitlabNamespaces for HttpGitlabNamespaces {
    async fn lookup(&self, endpoint: &TrackerEndpoint) -> Result<Option<GitlabNamespace>, TrackerFailure> {
        let encoded = gitlab_project(&endpoint.location);
        if let Some(project) = self.read::<GitlabProject>(endpoint, &format!("/projects/{encoded}"), &[]).await? {
            return Ok(Some(project.into()));
        }
        match self.read::<GitlabGroup>(endpoint, &format!("/groups/{encoded}"), &[]).await? {
            Some(group) => self.group_with_totals(endpoint, group).await.map(Some),
            None => Ok(None),
        }
    }

    async fn children(&self, endpoint: &TrackerEndpoint, query: &str, limit: u16) -> Result<Vec<GitlabNamespace>, TrackerFailure> {
        let filter = [("search", query.to_string()), ("per_page", limit.to_string())];
        if endpoint.location.is_empty() {
            let top_level = [filter.as_slice(), &[("top_level_only", "true".to_string())]].concat();
            let groups = self.read::<Vec<GitlabGroup>>(endpoint, "/groups", &top_level).await?.unwrap_or_default();
            return Ok(groups.into_iter().map(GitlabNamespace::from).collect());
        }
        let encoded = gitlab_project(&endpoint.location);
        let subgroups = self.read::<Vec<GitlabGroup>>(endpoint, &format!("/groups/{encoded}/subgroups"), &filter).await?.unwrap_or_default();
        let projects = self.read::<Vec<GitlabProject>>(endpoint, &format!("/groups/{encoded}/projects"), &filter).await?.unwrap_or_default();
        let found = subgroups.into_iter().map(GitlabNamespace::from).chain(projects.into_iter().map(GitlabNamespace::from));
        Ok(found.take(usize::from(limit)).collect())
    }
}

#[cfg(test)]
#[path = "../../tests/unit/tracker/http_gitlab_namespaces_tests.rs"]
mod tests;
