use std::time::Duration;

use async_trait::async_trait;
use reqwest::header::USER_AGENT;
use reqwest::{Method, Response, StatusCode};
use rekall_core::connection::{Mergeability, PullRequest, PullRequestHost, PullRequestMerge, TrackerEndpoint, TrackerFailure};
use rekall_store::ConnectionKind;
use serde::Deserialize;
use serde_json::{json, Value};

use super::http_task_tracker::{checked, gitlab_project, said_by, signature_of, Refusal};
use crate::probes::{client_through, failure_chain};

const TIMEOUT: Duration = Duration::from_secs(15);
const CHECK_INTERVAL: Duration = Duration::from_secs(1);
const CHECK_ATTEMPTS: usize = 5;
const MERGEABLE_STATUS: &str = "can_be_merged";
const CONFLICTING_STATUS: &str = "cannot_be_merged";
const CONFLICTING_RECHECK_STATUS: &str = "cannot_be_merged_recheck";

#[derive(Deserialize)]
struct GitlabMergeRequest {
    iid: u64,
    web_url: String,
    merge_status: Option<String>,
}

/// GitLab's merge request API over HTTPS with `reqwest`, through the connection's proxy when it
/// names one. GitLab computes mergeability after the request is read, so a status still `checking`
/// is read again a few times before it is `Unknown`.
pub struct GitlabPullRequestHost {
    timeout: Duration,
    check_interval: Duration,
}

impl GitlabPullRequestHost {
    pub fn new() -> Self {
        Self { timeout: TIMEOUT, check_interval: CHECK_INTERVAL }
    }

    async fn send(&self, endpoint: &TrackerEndpoint, method: Method, path: &str, query: &[(&str, String)], body: Option<Value>) -> Result<Response, TrackerFailure> {
        require_gitlab(endpoint)?;
        let client = client_through(endpoint.proxy_url.as_deref(), self.timeout).map_err(|reason| TrackerFailure::Failed { reason })?;
        let (name, value) = signature_of(endpoint)?;
        let site = endpoint.site_url.trim_end_matches('/');
        let mut request = client
            .request(method, format!("{site}/api/v4/projects/{}{path}", gitlab_project(&endpoint.location)))
            .header(name, value)
            .header(USER_AGENT, "rekall")
            .query(query);
        if let Some(body) = body {
            request = request.json(&body);
        }
        request.send().await.map_err(|failure| TrackerFailure::Failed { reason: failure_chain(&failure) })
    }

    async fn read(&self, endpoint: &TrackerEndpoint, iid: u64) -> Result<PullRequest, TrackerFailure> {
        let mut merge_request = self.read_once(endpoint, iid).await?;
        for _ in 1..CHECK_ATTEMPTS {
            if mergeability_of(&merge_request) != Mergeability::Unknown {
                break;
            }
            tokio::time::sleep(self.check_interval).await;
            merge_request = self.read_once(endpoint, iid).await?;
        }
        Ok(pull_request_of(merge_request))
    }

    async fn read_once(&self, endpoint: &TrackerEndpoint, iid: u64) -> Result<GitlabMergeRequest, TrackerFailure> {
        let response = checked(self.send(endpoint, Method::GET, &format!("/merge_requests/{iid}"), &[], None).await?).await?;
        response.json::<GitlabMergeRequest>().await.map_err(unreadable)
    }

    async fn refusal_of_merge(&self, endpoint: &TrackerEndpoint, pull_request: &PullRequest, response: Response) -> Result<PullRequestMerge, TrackerFailure> {
        let reason = response.json::<Refusal>().await.ok().map(|refusal| said_by(&refusal)).unwrap_or_default();
        if self.read(endpoint, pull_request.number).await?.mergeability == Mergeability::Conflicting {
            return Ok(PullRequestMerge::Conflicting);
        }
        Ok(PullRequestMerge::Refused(if reason.is_empty() { "GitLab refused the merge".to_string() } else { reason }))
    }
}

impl Default for GitlabPullRequestHost {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PullRequestHost for GitlabPullRequestHost {
    async fn open(&self, endpoint: &TrackerEndpoint, head: &str, base: &str, title: &str, body: &str) -> Result<PullRequest, TrackerFailure> {
        if let Some(existing) = self.find(endpoint, head).await? {
            return Ok(existing);
        }
        let payload = json!({ "source_branch": head, "target_branch": base, "title": title, "description": body });
        let response = checked(self.send(endpoint, Method::POST, "/merge_requests", &[], Some(payload)).await?).await?;
        let created = response.json::<GitlabMergeRequest>().await.map_err(unreadable)?;
        Ok(pull_request_of(created))
    }

    async fn find(&self, endpoint: &TrackerEndpoint, head: &str) -> Result<Option<PullRequest>, TrackerFailure> {
        let query = [("source_branch", head.to_string()), ("state", "opened".to_string())];
        let response = checked(self.send(endpoint, Method::GET, "/merge_requests", &query, None).await?).await?;
        let listed = response.json::<Vec<GitlabMergeRequest>>().await.map_err(unreadable)?;
        match listed.first() {
            Some(merge_request) => self.read(endpoint, merge_request.iid).await.map(Some),
            None => Ok(None),
        }
    }

    async fn merge(&self, endpoint: &TrackerEndpoint, pull_request: &PullRequest) -> Result<PullRequestMerge, TrackerFailure> {
        let response = self.send(endpoint, Method::PUT, &format!("/merge_requests/{}/merge", pull_request.number), &[], Some(json!({ "squash": false }))).await?;
        match response.status() {
            StatusCode::METHOD_NOT_ALLOWED | StatusCode::NOT_ACCEPTABLE | StatusCode::CONFLICT => self.refusal_of_merge(endpoint, pull_request, response).await,
            _ => checked(response).await.map(|_| PullRequestMerge::Merged),
        }
    }

    async fn close_issue(&self, endpoint: &TrackerEndpoint, issue_key: &str) -> Result<(), TrackerFailure> {
        let response = self.send(endpoint, Method::PUT, &format!("/issues/{issue_key}"), &[], Some(json!({ "state_event": "close" }))).await?;
        checked(response).await.map(|_| ())
    }

    async fn delete_branch(&self, endpoint: &TrackerEndpoint, branch: &str) -> Result<(), TrackerFailure> {
        let response = self.send(endpoint, Method::DELETE, &format!("/repository/branches/{}", branch.replace('/', "%2F")), &[], None).await?;
        match response.status() {
            StatusCode::NOT_FOUND => Ok(()),
            _ => checked(response).await.map(|_| ()),
        }
    }
}

fn require_gitlab(endpoint: &TrackerEndpoint) -> Result<(), TrackerFailure> {
    if endpoint.kind == ConnectionKind::Gitlab {
        return Ok(());
    }
    Err(TrackerFailure::Failed { reason: "only a GitLab connection takes merge requests here".into() })
}

fn pull_request_of(merge_request: GitlabMergeRequest) -> PullRequest {
    let mergeability = mergeability_of(&merge_request);
    PullRequest { number: merge_request.iid, url: merge_request.web_url, mergeability }
}

fn mergeability_of(merge_request: &GitlabMergeRequest) -> Mergeability {
    match merge_request.merge_status.as_deref() {
        Some(MERGEABLE_STATUS) => Mergeability::Mergeable,
        Some(CONFLICTING_STATUS | CONFLICTING_RECHECK_STATUS) => Mergeability::Conflicting,
        _ => Mergeability::Unknown,
    }
}

fn unreadable(failure: reqwest::Error) -> TrackerFailure {
    TrackerFailure::Failed { reason: format!("its answer did not read as a merge request ({failure})") }
}

#[cfg(test)]
#[path = "../../tests/unit/tracker/gitlab_pull_request_host_tests.rs"]
mod tests;
