use std::time::Duration;

use async_trait::async_trait;
use reqwest::header::{ACCEPT, USER_AGENT};
use reqwest::{Method, Response, StatusCode};
use rekall_core::connection::{github_api_root, Mergeability, PullRequest, PullRequestHost, PullRequestMerge, TrackerEndpoint, TrackerFailure};
use rekall_store::ConnectionKind;
use serde::Deserialize;
use serde_json::{json, Value};

use super::http_task_tracker::{checked, said_by, signature_of, Refusal};
use crate::probes::{client_through, failure_chain};

const TIMEOUT: Duration = Duration::from_secs(15);
const CONFLICTING_STATE: &str = "dirty";

#[derive(Deserialize)]
struct GithubPull {
    number: u64,
    html_url: String,
    mergeable: Option<bool>,
    mergeable_state: Option<String>,
}

/// GitHub's pull request API over HTTPS with `reqwest`, through the connection's proxy when it
/// names one. A merge GitHub answers with 405 or 409 is `Conflicting` when the pull request reads
/// back as conflicting and `Refused` with GitHub's message otherwise.
pub struct GithubPullRequestHost {
    timeout: Duration,
}

impl GithubPullRequestHost {
    pub fn new() -> Self {
        Self { timeout: TIMEOUT }
    }

    async fn send(&self, endpoint: &TrackerEndpoint, method: Method, path: &str, query: &[(&str, String)], body: Option<Value>) -> Result<Response, TrackerFailure> {
        require_github(endpoint)?;
        let client = client_through(endpoint.proxy_url.as_deref(), self.timeout).map_err(|reason| TrackerFailure::Failed { reason })?;
        let (name, value) = signature_of(endpoint)?;
        let root = github_api_root(endpoint.site_url.trim_end_matches('/'));
        let mut request = client
            .request(method, format!("{root}/repos/{}{path}", endpoint.location))
            .header(name, value)
            .header(USER_AGENT, "rekall")
            .header(ACCEPT, "application/vnd.github+json")
            .query(query);
        if let Some(body) = body {
            request = request.json(&body);
        }
        request.send().await.map_err(|failure| TrackerFailure::Failed { reason: failure_chain(&failure) })
    }

    async fn read(&self, endpoint: &TrackerEndpoint, number: u64) -> Result<PullRequest, TrackerFailure> {
        let response = checked(self.send(endpoint, Method::GET, &format!("/pulls/{number}"), &[], None).await?).await?;
        pull_request_of(response).await
    }

    async fn refusal_of_merge(&self, endpoint: &TrackerEndpoint, pull_request: &PullRequest, response: Response) -> Result<PullRequestMerge, TrackerFailure> {
        let reason = response.json::<Refusal>().await.ok().map(|refusal| said_by(&refusal)).unwrap_or_default();
        if self.read(endpoint, pull_request.number).await?.mergeability == Mergeability::Conflicting {
            return Ok(PullRequestMerge::Conflicting);
        }
        Ok(PullRequestMerge::Refused(if reason.is_empty() { "GitHub refused the merge".to_string() } else { reason }))
    }
}

impl Default for GithubPullRequestHost {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PullRequestHost for GithubPullRequestHost {
    async fn open(&self, endpoint: &TrackerEndpoint, head: &str, base: &str, title: &str, body: &str) -> Result<PullRequest, TrackerFailure> {
        if let Some(existing) = self.find(endpoint, head).await? {
            return Ok(existing);
        }
        let payload = json!({ "title": title, "head": head, "base": base, "body": body });
        let response = checked(self.send(endpoint, Method::POST, "/pulls", &[], Some(payload)).await?).await?;
        pull_request_of(response).await
    }

    async fn find(&self, endpoint: &TrackerEndpoint, head: &str) -> Result<Option<PullRequest>, TrackerFailure> {
        let query = [("head", format!("{}:{head}", owner_of(endpoint))), ("state", "open".to_string())];
        let response = checked(self.send(endpoint, Method::GET, "/pulls", &query, None).await?).await?;
        let listed = response.json::<Vec<GithubPull>>().await.map_err(unreadable)?;
        match listed.first() {
            Some(pull) => self.read(endpoint, pull.number).await.map(Some),
            None => Ok(None),
        }
    }

    async fn merge(&self, endpoint: &TrackerEndpoint, pull_request: &PullRequest) -> Result<PullRequestMerge, TrackerFailure> {
        let response = self.send(endpoint, Method::PUT, &format!("/pulls/{}/merge", pull_request.number), &[], Some(json!({ "merge_method": "merge" }))).await?;
        match response.status() {
            StatusCode::METHOD_NOT_ALLOWED | StatusCode::CONFLICT => self.refusal_of_merge(endpoint, pull_request, response).await,
            _ => checked(response).await.map(|_| PullRequestMerge::Merged),
        }
    }

    async fn close_issue(&self, endpoint: &TrackerEndpoint, issue_key: &str) -> Result<(), TrackerFailure> {
        let response = self.send(endpoint, Method::PATCH, &format!("/issues/{issue_key}"), &[], Some(json!({ "state": "closed" }))).await?;
        checked(response).await.map(|_| ())
    }
}

fn require_github(endpoint: &TrackerEndpoint) -> Result<(), TrackerFailure> {
    if endpoint.kind == ConnectionKind::Github {
        return Ok(());
    }
    Err(TrackerFailure::Failed { reason: "only a GitHub connection takes pull requests here".into() })
}

fn owner_of(endpoint: &TrackerEndpoint) -> &str {
    endpoint.location.split('/').next().unwrap_or_default()
}

async fn pull_request_of(response: Response) -> Result<PullRequest, TrackerFailure> {
    let pull = response.json::<GithubPull>().await.map_err(unreadable)?;
    let mergeability = mergeability_of(&pull);
    Ok(PullRequest { number: pull.number, url: pull.html_url, mergeability })
}

fn mergeability_of(pull: &GithubPull) -> Mergeability {
    if pull.mergeable_state.as_deref() == Some(CONFLICTING_STATE) || pull.mergeable == Some(false) {
        return Mergeability::Conflicting;
    }
    match pull.mergeable {
        Some(true) => Mergeability::Mergeable,
        _ => Mergeability::Unknown,
    }
}

fn unreadable(failure: reqwest::Error) -> TrackerFailure {
    TrackerFailure::Failed { reason: format!("its answer did not read as a pull request ({failure})") }
}

#[cfg(test)]
#[path = "../../tests/unit/tracker/github_pull_request_host_tests.rs"]
mod tests;
