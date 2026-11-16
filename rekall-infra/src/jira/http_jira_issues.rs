use std::time::Duration;

use async_trait::async_trait;
use reqwest::header::{HeaderValue, ACCEPT, AUTHORIZATION};
use reqwest::{Client, Response, StatusCode};
use rekall_core::connection::{AtlassianCredential, AtlassianDeployment, AtlassianFailure, AtlassianSite, JiraIssue, JiraIssues, JiraProject};
use serde::Deserialize;
use serde_json::json;

use super::atlassian_document::AdfNode;
use crate::probes::{client_through, failure_chain};

const TIMEOUT: Duration = Duration::from_secs(15);
const CLOUD_SEARCH: &str = "/rest/api/3/search/jql";
const DATA_CENTER_SEARCH: &str = "/rest/api/2/search";
const CLOUD_PROJECTS: &str = "/rest/api/3/project/search";
const DATA_CENTER_PROJECTS: &str = "/rest/api/2/project";
const FIELDS: [&str; 4] = ["summary", "status", "issuetype", "description"];

#[derive(Deserialize)]
struct SearchAnswer {
    #[serde(default)]
    issues: Vec<FoundIssue>,
}

/// Cloud pages its projects under `values`; Data Center answers them as a bare list.
#[derive(Deserialize)]
#[serde(untagged)]
enum ProjectListing {
    Page { values: Vec<FoundProject> },
    List(Vec<FoundProject>),
}

#[derive(Deserialize)]
struct FoundProject {
    key: String,
    #[serde(default)]
    name: String,
}

#[derive(Deserialize)]
struct FoundIssue {
    key: String,
    fields: IssueFields,
}

#[derive(Deserialize)]
struct IssueFields {
    #[serde(default)]
    summary: String,
    status: Option<Named>,
    issuetype: Option<Named>,
    description: Option<Description>,
}

/// Cloud's v3 API answers a description as Atlassian's document tree; Data Center's v2 as its text.
#[derive(Deserialize)]
#[serde(untagged)]
enum Description {
    Text(String),
    Document(AdfNode),
}

#[derive(Deserialize)]
struct Named {
    name: Option<String>,
}

/// What Jira says when it refuses a request.
#[derive(Deserialize)]
struct JiraErrors {
    #[serde(rename = "errorMessages", default)]
    error_messages: Vec<String>,
}

/// Jira's REST API over HTTPS with `reqwest`, Cloud's v3 or Data Center's v2 as the credential says,
/// through the connection's proxy when it names one: one search, the first page only.
pub struct HttpJiraIssues {
    timeout: Duration,
}

impl HttpJiraIssues {
    pub fn new() -> Self {
        Self { timeout: TIMEOUT }
    }
}

impl Default for HttpJiraIssues {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl JiraIssues for HttpJiraIssues {
    async fn search(&self, site: &AtlassianSite, credential: &AtlassianCredential, jql: &str, limit: u16) -> Result<Vec<JiraIssue>, AtlassianFailure> {
        let client = jira_client(site, self.timeout).map_err(|reason| AtlassianFailure::Failed { reason })?;
        let body = json!({ "jql": jql, "maxResults": limit, "fields": FIELDS });
        let answer = client
            .post(format!("{}{}", site.url.trim_end_matches('/'), search_path(credential.deployment())))
            .header(AUTHORIZATION, authorization(credential)?)
            .header(ACCEPT, "application/json")
            .json(&body)
            .send()
            .await;
        let response = accepted(answer).await?;
        let found = response.json::<SearchAnswer>().await.map_err(|failure| AtlassianFailure::Failed { reason: format!("its answer did not read as a search result ({failure})") })?;
        Ok(found.issues.into_iter().map(issue_of).collect())
    }

    async fn projects(&self, site: &AtlassianSite, credential: &AtlassianCredential, query: &str, limit: u16) -> Result<Vec<JiraProject>, AtlassianFailure> {
        let client = jira_client(site, self.timeout).map_err(|reason| AtlassianFailure::Failed { reason })?;
        let path = match credential.deployment() {
            AtlassianDeployment::Cloud => CLOUD_PROJECTS,
            AtlassianDeployment::DataCenter => DATA_CENTER_PROJECTS,
        };
        let answer = client
            .get(format!("{}{path}", site.url.trim_end_matches('/')))
            .query(&[("query", query.trim()), ("maxResults", &limit.to_string())])
            .header(AUTHORIZATION, authorization(credential)?)
            .header(ACCEPT, "application/json")
            .send()
            .await;
        let listing = accepted(answer).await?.json::<ProjectListing>().await.map_err(|failure| AtlassianFailure::Failed { reason: format!("its answer did not read as a project list ({failure})") })?;
        let found = match listing {
            ProjectListing::Page { values } => values,
            ProjectListing::List(list) => list,
        };
        let wanted = query.trim().to_lowercase();
        let matching = found.into_iter().filter(|project| wanted.is_empty() || project.key.to_lowercase().contains(&wanted) || project.name.to_lowercase().contains(&wanted));
        Ok(matching.take(usize::from(limit)).map(|project| JiraProject { key: project.key, name: project.name }).collect())
    }

    async fn project(&self, site: &AtlassianSite, credential: &AtlassianCredential, key: &str) -> Result<Option<JiraProject>, AtlassianFailure> {
        let client = jira_client(site, self.timeout).map_err(|reason| AtlassianFailure::Failed { reason })?;
        let answer = client
            .get(format!("{}{}/project/{key}", site.url.trim_end_matches('/'), api_root(credential.deployment())))
            .header(AUTHORIZATION, authorization(credential)?)
            .header(ACCEPT, "application/json")
            .send()
            .await;
        let response = match answer {
            Ok(response) if response.status() == StatusCode::NOT_FOUND => return Ok(None),
            answer => accepted(answer).await?,
        };
        let found = response.json::<FoundProject>().await.map_err(|failure| AtlassianFailure::Failed { reason: format!("its answer did not read as a project ({failure})") })?;
        Ok(Some(JiraProject { key: found.key, name: found.name }))
    }
}

pub(crate) fn jira_client(site: &AtlassianSite, timeout: Duration) -> Result<Client, String> {
    client_through(site.proxy_url.as_deref(), timeout)
}

pub(crate) fn authorization(credential: &AtlassianCredential) -> Result<HeaderValue, AtlassianFailure> {
    let mut header = HeaderValue::from_str(credential.authorization().expose())
        .map_err(|_| AtlassianFailure::Failed { reason: "the stored credential holds characters a header cannot carry".into() })?;
    header.set_sensitive(true);
    Ok(header)
}

fn api_root(deployment: AtlassianDeployment) -> &'static str {
    match deployment {
        AtlassianDeployment::Cloud => "/rest/api/3",
        AtlassianDeployment::DataCenter => "/rest/api/2",
    }
}

fn search_path(deployment: AtlassianDeployment) -> &'static str {
    match deployment {
        AtlassianDeployment::Cloud => CLOUD_SEARCH,
        AtlassianDeployment::DataCenter => DATA_CENTER_SEARCH,
    }
}

fn issue_of(found: FoundIssue) -> JiraIssue {
    let description = found.fields.description.map(markdown_of).filter(|text| !text.is_empty());
    JiraIssue {
        key: found.key,
        summary: found.fields.summary,
        status: found.fields.status.and_then(|status| status.name),
        issue_type: found.fields.issuetype.and_then(|kind| kind.name),
        description,
    }
}

fn markdown_of(description: Description) -> String {
    match description {
        Description::Text(text) => text.trim().to_string(),
        Description::Document(document) => document.to_markdown(),
    }
}

/// The response when Jira answered the search, or why it did not, in Jira's own words when it gave some.
async fn accepted(answer: reqwest::Result<Response>) -> Result<Response, AtlassianFailure> {
    let response = answer.map_err(|failure| AtlassianFailure::Failed { reason: failure_chain(&failure) })?;
    match response.status() {
        status if status.is_success() => Ok(response),
        status @ (StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN) => Err(AtlassianFailure::Rejected { status: status.as_u16() }),
        status => {
            let messages = response.json::<JiraErrors>().await.map(|errors| errors.error_messages.join(" ")).unwrap_or_default();
            let reason = if messages.is_empty() { format!("it answered HTTP {}", status.as_u16()) } else { format!("it answered HTTP {}: {messages}", status.as_u16()) };
            Err(AtlassianFailure::Failed { reason })
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/jira/http_jira_issues_tests.rs"]
mod tests;
