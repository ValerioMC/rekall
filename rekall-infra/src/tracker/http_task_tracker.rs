use std::time::Duration;

use async_trait::async_trait;
use reqwest::header::{HeaderName, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use reqwest::{Method, Response, StatusCode};
use rekall_core::connection::{github_api_root, JiraCredential, JiraDeployment, TaskTracker, TrackerEndpoint, TrackerFailure, TrackerIssue, TrackerLink};
use rekall_store::ConnectionKind;
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::warn;

use crate::probes::{client_through, failure_chain};

const TIMEOUT: Duration = Duration::from_secs(15);
const REASON_MAX_CHARACTERS: usize = 300;
const JIRA_ISSUE_TYPE: &str = "Task";
const GITLAB_ISSUE_TYPE: &str = "task";
const GITHUB_ISSUE_TYPE: &str = "Task";
const DELETE_ISSUE_MUTATION: &str = "mutation($id: ID!) { deleteIssue(input: { issueId: $id }) { clientMutationId } }";

#[derive(Deserialize)]
struct JiraCreated {
    key: String,
}

#[derive(Deserialize)]
struct GitlabIssue {
    iid: u64,
    web_url: String,
}

#[derive(Deserialize)]
struct GithubNode {
    node_id: String,
}

#[derive(Deserialize)]
struct GraphqlAnswer {
    #[serde(default)]
    errors: Vec<Value>,
}

#[derive(Deserialize)]
struct GithubIssue {
    number: u64,
    html_url: String,
}

/// What a tracker says when it refuses a request, in whichever of its shapes.
#[derive(Deserialize)]
pub(super) struct Refusal {
    message: Option<Value>,
    #[serde(rename = "errorMessages", default)]
    error_messages: Vec<String>,
    #[serde(default)]
    errors: Option<Value>,
}

/// One request to a tracker: where, with what body, and the header that signs it.
struct Call {
    method: Method,
    url: String,
    body: Value,
    signature: (HeaderName, HeaderValue),
}

/// GitLab's, GitHub's and Jira's REST APIs over HTTPS with `reqwest`: an issue is created with a
/// POST, assigned to the user the credential belongs to and typed as a task, and rewritten with a
/// PUT (GitHub: PATCH), through the connection's proxy when it names one.
pub struct HttpTaskTracker {
    timeout: Duration,
}

impl HttpTaskTracker {
    pub fn new() -> Self {
        Self { timeout: TIMEOUT }
    }

    async fn send(&self, endpoint: &TrackerEndpoint, call: Call) -> Result<Response, TrackerFailure> {
        let client = client_through(endpoint.proxy_url.as_deref(), self.timeout).map_err(|reason| TrackerFailure::Failed { reason })?;
        let (name, value) = call.signature;
        let mut request = client.request(call.method, call.url).header(name, value).header(USER_AGENT, "rekall").header(ACCEPT, "application/json");
        if !call.body.is_null() {
            request = request.json(&call.body);
        }
        accepted(request.send().await).await
    }

    /// The assignee value of the user the credential belongs to, in the shape the tracker's create
    /// call takes. An issue is better created unassigned than not at all, so a lookup that fails is none.
    async fn assignee_of(&self, endpoint: &TrackerEndpoint) -> Option<Value> {
        let looked_up = match self.send(endpoint, current_user_call(endpoint).ok()?).await {
            Ok(response) => response.json::<Value>().await.map_err(|failure| TrackerFailure::Failed { reason: failure.to_string() }),
            Err(failure) => Err(failure),
        };
        match looked_up {
            Ok(user) => assignee_in(endpoint, &user),
            Err(failure) => {
                warn!(?failure, "The task manager did not say who the credential belongs to; the issue is created unassigned");
                None
            }
        }
    }

    async fn delete_github_issue(&self, endpoint: &TrackerEndpoint, key: &str) -> Result<(), TrackerFailure> {
        let root = github_api_root(endpoint.site_url.trim_end_matches('/'));
        let signature = signature_of(endpoint)?;
        let issue = self.send(endpoint, Call { method: Method::GET, url: format!("{root}/repos/{}/issues/{key}", endpoint.location), body: Value::Null, signature: signature.clone() }).await?;
        let node_id = issue.json::<GithubNode>().await.map_err(|failure| TrackerFailure::Failed { reason: format!("its answer did not read as an issue ({failure})") })?.node_id;
        let mutation = json!({ "query": DELETE_ISSUE_MUTATION, "variables": { "id": node_id } });
        let answer = self.send(endpoint, Call { method: Method::POST, url: github_graphql_url(&root), body: mutation, signature: signature.clone() }).await?;
        let deleted = answer.json::<GraphqlAnswer>().await.is_ok_and(|answer| answer.errors.is_empty());
        if deleted {
            return Ok(());
        }
        let closed = json!({ "state": "closed", "state_reason": "not_planned" });
        self.send(endpoint, Call { method: Method::PATCH, url: format!("{root}/repos/{}/issues/{key}", endpoint.location), body: closed, signature }).await.map(|_| ())
    }
}

impl Default for HttpTaskTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TaskTracker for HttpTaskTracker {
    async fn create(&self, endpoint: &TrackerEndpoint, issue: &TrackerIssue) -> Result<TrackerLink, TrackerFailure> {
        let assignee = self.assignee_of(endpoint).await;
        let response = match self.send(endpoint, create_call(endpoint, issue, assignee.as_ref(), true)?).await {
            Err(TrackerFailure::Failed { reason }) if endpoint.kind != ConnectionKind::Jira => {
                warn!(%reason, "The task manager refused the issue type; the issue is created without it");
                self.send(endpoint, create_call(endpoint, issue, assignee.as_ref(), false)?).await?
            }
            answered => answered?,
        };
        link_of(endpoint, response).await
    }

    async fn delete(&self, endpoint: &TrackerEndpoint, key: &str) -> Result<(), TrackerFailure> {
        if endpoint.kind == ConnectionKind::Github {
            return self.delete_github_issue(endpoint, key).await;
        }
        self.send(endpoint, delete_call(endpoint, key)?).await.map(|_| ())
    }

    async fn update(&self, endpoint: &TrackerEndpoint, key: &str, issue: &TrackerIssue) -> Result<(), TrackerFailure> {
        self.send(endpoint, update_call(endpoint, key, issue)?).await.map(|_| ())
    }
}

fn create_call(endpoint: &TrackerEndpoint, issue: &TrackerIssue, assignee: Option<&Value>, typed: bool) -> Result<Call, TrackerFailure> {
    let site = endpoint.site_url.trim_end_matches('/');
    let signature = signature_of(endpoint)?;
    match endpoint.kind {
        ConnectionKind::Jira => {
            let deployment = jira_deployment(endpoint)?;
            let mut fields = json!({
                "project": { "key": endpoint.location },
                "summary": issue.title,
                "description": jira_description(deployment, &issue.description),
                "issuetype": { "name": JIRA_ISSUE_TYPE },
            });
            if let Some(assignee) = assignee {
                fields["assignee"] = assignee.clone();
            }
            Ok(Call { method: Method::POST, url: format!("{site}{}/issue", jira_api(deployment)), body: json!({ "fields": fields }), signature })
        }
        ConnectionKind::Gitlab => {
            let mut body = json!({ "title": issue.title, "description": issue.description });
            if let Some(assignee) = assignee {
                body["assignee_ids"] = assignee.clone();
            }
            if typed {
                body["issue_type"] = json!(GITLAB_ISSUE_TYPE);
            }
            Ok(Call { method: Method::POST, url: format!("{site}/api/v4/projects/{}/issues", gitlab_project(&endpoint.location)), body, signature })
        }
        ConnectionKind::Github => {
            let mut body = json!({ "title": issue.title, "body": issue.description });
            if let Some(assignee) = assignee {
                body["assignees"] = assignee.clone();
            }
            if typed {
                body["type"] = json!(GITHUB_ISSUE_TYPE);
            }
            Ok(Call { method: Method::POST, url: format!("{}/repos/{}/issues", github_api_root(site), endpoint.location), body, signature })
        }
        ConnectionKind::Kubernetes | ConnectionKind::Grafana => Err(not_a_task_manager()),
    }
}

fn current_user_call(endpoint: &TrackerEndpoint) -> Result<Call, TrackerFailure> {
    let site = endpoint.site_url.trim_end_matches('/');
    let signature = signature_of(endpoint)?;
    let url = match endpoint.kind {
        ConnectionKind::Jira => format!("{site}{}/myself", jira_api(jira_deployment(endpoint)?)),
        ConnectionKind::Gitlab => format!("{site}/api/v4/user"),
        ConnectionKind::Github => format!("{}/user", github_api_root(site)),
        ConnectionKind::Kubernetes | ConnectionKind::Grafana => return Err(not_a_task_manager()),
    };
    Ok(Call { method: Method::GET, url, body: Value::Null, signature })
}

/// What each tracker names a person by: GitHub the login, GitLab the id, Jira Cloud the account id and Data Center the user name.
fn assignee_in(endpoint: &TrackerEndpoint, user: &Value) -> Option<Value> {
    match endpoint.kind {
        ConnectionKind::Github => user.get("login").map(|login| json!([login])),
        ConnectionKind::Gitlab => user.get("id").map(|id| json!([id])),
        ConnectionKind::Jira => match jira_deployment(endpoint).ok()? {
            JiraDeployment::Cloud => user.get("accountId").map(|account| json!({ "accountId": account })),
            JiraDeployment::DataCenter => user.get("name").map(|name| json!({ "name": name })),
        },
        ConnectionKind::Kubernetes | ConnectionKind::Grafana => None,
    }
}

fn delete_call(endpoint: &TrackerEndpoint, key: &str) -> Result<Call, TrackerFailure> {
    let site = endpoint.site_url.trim_end_matches('/');
    let signature = signature_of(endpoint)?;
    let url = match endpoint.kind {
        ConnectionKind::Jira => format!("{site}{}/issue/{key}?deleteSubtasks=true", jira_api(jira_deployment(endpoint)?)),
        ConnectionKind::Gitlab => format!("{site}/api/v4/projects/{}/issues/{key}", gitlab_project(&endpoint.location)),
        ConnectionKind::Github | ConnectionKind::Kubernetes | ConnectionKind::Grafana => return Err(not_a_task_manager()),
    };
    Ok(Call { method: Method::DELETE, url, body: Value::Null, signature })
}

/// GitHub deletes an issue only through its GraphQL API, and its answer to a refusal is a 200 with `errors`.
fn github_graphql_url(rest_root: &str) -> String {
    match rest_root.strip_suffix("/v3") {
        Some(api) => format!("{api}/graphql"),
        None => format!("{rest_root}/graphql"),
    }
}

fn update_call(endpoint: &TrackerEndpoint, key: &str, issue: &TrackerIssue) -> Result<Call, TrackerFailure> {
    let site = endpoint.site_url.trim_end_matches('/');
    let signature = signature_of(endpoint)?;
    match endpoint.kind {
        ConnectionKind::Jira => {
            let deployment = jira_deployment(endpoint)?;
            let fields = json!({ "summary": issue.title, "description": jira_description(deployment, &issue.description) });
            Ok(Call { method: Method::PUT, url: format!("{site}{}/issue/{key}", jira_api(deployment)), body: json!({ "fields": fields }), signature })
        }
        ConnectionKind::Gitlab => Ok(Call {
            method: Method::PUT,
            url: format!("{site}/api/v4/projects/{}/issues/{key}", gitlab_project(&endpoint.location)),
            body: json!({ "title": issue.title, "description": issue.description }),
            signature,
        }),
        ConnectionKind::Github => Ok(Call {
            method: Method::PATCH,
            url: format!("{}/repos/{}/issues/{key}", github_api_root(site), endpoint.location),
            body: json!({ "title": issue.title, "body": issue.description }),
            signature,
        }),
        ConnectionKind::Kubernetes | ConnectionKind::Grafana => Err(not_a_task_manager()),
    }
}

async fn link_of(endpoint: &TrackerEndpoint, response: Response) -> Result<TrackerLink, TrackerFailure> {
    let unreadable = |failure: reqwest::Error| TrackerFailure::Failed { reason: format!("its answer did not read as an issue ({failure})") };
    match endpoint.kind {
        ConnectionKind::Jira => {
            let created = response.json::<JiraCreated>().await.map_err(unreadable)?;
            let url = format!("{}/browse/{}", endpoint.site_url.trim_end_matches('/'), created.key);
            Ok(TrackerLink { key: created.key, url })
        }
        ConnectionKind::Gitlab => response.json::<GitlabIssue>().await.map(|issue| TrackerLink { key: issue.iid.to_string(), url: issue.web_url }).map_err(unreadable),
        ConnectionKind::Github => response.json::<GithubIssue>().await.map(|issue| TrackerLink { key: issue.number.to_string(), url: issue.html_url }).map_err(unreadable),
        ConnectionKind::Kubernetes | ConnectionKind::Grafana => Err(not_a_task_manager()),
    }
}

pub(super) fn signature_of(endpoint: &TrackerEndpoint) -> Result<(HeaderName, HeaderValue), TrackerFailure> {
    let (name, text) = match endpoint.kind {
        ConnectionKind::Gitlab => (HeaderName::from_static("private-token"), endpoint.credential.expose().to_string()),
        ConnectionKind::Github => (AUTHORIZATION, format!("Bearer {}", endpoint.credential.expose())),
        ConnectionKind::Jira => (AUTHORIZATION, jira_credential(endpoint)?.authorization().expose().to_string()),
        ConnectionKind::Kubernetes | ConnectionKind::Grafana => return Err(not_a_task_manager()),
    };
    let mut value = HeaderValue::from_str(&text).map_err(|_| TrackerFailure::Failed { reason: "the stored credential holds characters a header cannot carry".into() })?;
    value.set_sensitive(true);
    Ok((name, value))
}

fn jira_credential(endpoint: &TrackerEndpoint) -> Result<JiraCredential, TrackerFailure> {
    JiraCredential::from_stored(&endpoint.credential).map_err(|failure| TrackerFailure::Failed { reason: failure.message().to_string() })
}

fn jira_deployment(endpoint: &TrackerEndpoint) -> Result<JiraDeployment, TrackerFailure> {
    jira_credential(endpoint).map(|credential| credential.deployment())
}

fn jira_api(deployment: JiraDeployment) -> &'static str {
    match deployment {
        JiraDeployment::Cloud => "/rest/api/3",
        JiraDeployment::DataCenter => "/rest/api/2",
    }
}

/// Cloud's v3 API takes a description as Atlassian's document tree, one paragraph per block of
/// text; Data Center's v2 takes the text as it is.
fn jira_description(deployment: JiraDeployment, markdown: &str) -> Value {
    match deployment {
        JiraDeployment::DataCenter => Value::String(markdown.to_string()),
        JiraDeployment::Cloud => {
            let paragraphs: Vec<Value> = markdown
                .split("\n\n")
                .map(str::trim)
                .filter(|block| !block.is_empty())
                .map(|block| json!({ "type": "paragraph", "content": [{ "type": "text", "text": block }] }))
                .collect();
            json!({ "type": "doc", "version": 1, "content": paragraphs })
        }
    }
}

/// GitLab names a project in the path by its full path with the slashes encoded.
pub(super) fn gitlab_project(path: &str) -> String {
    path.replace('/', "%2F")
}

fn not_a_task_manager() -> TrackerFailure {
    TrackerFailure::Failed { reason: "only a GitLab, GitHub or Jira connection is a task manager".into() }
}

/// The response when the tracker took the issue, or why it did not, in its own words when it gave some.
async fn accepted(answer: reqwest::Result<Response>) -> Result<Response, TrackerFailure> {
    checked(answer.map_err(|failure| TrackerFailure::Failed { reason: failure_chain(&failure) })?).await
}

pub(super) async fn checked(response: Response) -> Result<Response, TrackerFailure> {
    match response.status() {
        status if status.is_success() => Ok(response),
        status @ (StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN) => Err(TrackerFailure::Rejected { status: status.as_u16() }),
        status => {
            let said = response.json::<Refusal>().await.ok().map(|refusal| said_by(&refusal)).unwrap_or_default();
            let reason = if said.is_empty() { format!("it answered HTTP {}", status.as_u16()) } else { format!("it answered HTTP {}: {said}", status.as_u16()) };
            Err(TrackerFailure::Failed { reason })
        }
    }
}

pub(super) fn said_by(refusal: &Refusal) -> String {
    let mut parts: Vec<String> = refusal.error_messages.clone();
    for detail in [refusal.message.as_ref(), refusal.errors.as_ref()].into_iter().flatten() {
        parts.push(match detail {
            Value::String(text) => text.clone(),
            other => other.to_string(),
        });
    }
    parts.join(" ").chars().take(REASON_MAX_CHARACTERS).collect()
}

#[cfg(test)]
#[path = "../../tests/unit/tracker/http_task_tracker_tests.rs"]
mod tests;
