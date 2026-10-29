use std::time::Duration;

use async_trait::async_trait;
use reqwest::header::{HeaderValue, ACCEPT, AUTHORIZATION};
use reqwest::{Client, Response, StatusCode};
use rekall_core::connection::{JiraCredential, JiraDeployment, JiraFailure, JiraIssue, JiraIssues, JiraSite};
use serde::Deserialize;
use serde_json::json;

use super::atlassian_document::AdfNode;
use crate::probes::failure_chain;

const TIMEOUT: Duration = Duration::from_secs(15);
const CLOUD_SEARCH: &str = "/rest/api/3/search/jql";
const DATA_CENTER_SEARCH: &str = "/rest/api/2/search";
const FIELDS: [&str; 4] = ["summary", "status", "issuetype", "description"];

#[derive(Deserialize)]
struct SearchAnswer {
    #[serde(default)]
    issues: Vec<FoundIssue>,
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
    async fn search(&self, site: &JiraSite, credential: &JiraCredential, jql: &str, limit: u16) -> Result<Vec<JiraIssue>, JiraFailure> {
        let client = jira_client(site, self.timeout).map_err(|reason| JiraFailure::Failed { reason })?;
        let body = json!({ "jql": jql, "maxResults": limit, "fields": FIELDS });
        let answer = client
            .post(format!("{}{}", site.url.trim_end_matches('/'), search_path(credential.deployment())))
            .header(AUTHORIZATION, authorization(credential)?)
            .header(ACCEPT, "application/json")
            .json(&body)
            .send()
            .await;
        let response = accepted(answer).await?;
        let found = response.json::<SearchAnswer>().await.map_err(|failure| JiraFailure::Failed { reason: format!("its answer did not read as a search result ({failure})") })?;
        Ok(found.issues.into_iter().map(issue_of).collect())
    }
}

pub(crate) fn jira_client(site: &JiraSite, timeout: Duration) -> Result<Client, String> {
    let mut builder = Client::builder().timeout(timeout);
    if let Some(proxy) = &site.proxy_url {
        builder = builder.proxy(reqwest::Proxy::all(proxy).map_err(|_| "the connection's proxy is not a usable proxy address".to_string())?);
    }
    builder.build().map_err(|failure| failure_chain(&failure))
}

pub(crate) fn authorization(credential: &JiraCredential) -> Result<HeaderValue, JiraFailure> {
    let mut header = HeaderValue::from_str(credential.authorization().expose())
        .map_err(|_| JiraFailure::Failed { reason: "the stored credential holds characters a header cannot carry".into() })?;
    header.set_sensitive(true);
    Ok(header)
}

fn search_path(deployment: JiraDeployment) -> &'static str {
    match deployment {
        JiraDeployment::Cloud => CLOUD_SEARCH,
        JiraDeployment::DataCenter => DATA_CENTER_SEARCH,
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
async fn accepted(answer: reqwest::Result<Response>) -> Result<Response, JiraFailure> {
    let response = answer.map_err(|failure| JiraFailure::Failed { reason: failure_chain(&failure) })?;
    match response.status() {
        status if status.is_success() => Ok(response),
        status @ (StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN) => Err(JiraFailure::Rejected { status: status.as_u16() }),
        status => {
            let messages = response.json::<JiraErrors>().await.map(|errors| errors.error_messages.join(" ")).unwrap_or_default();
            let reason = if messages.is_empty() { format!("it answered HTTP {}", status.as_u16()) } else { format!("it answered HTTP {}: {messages}", status.as_u16()) };
            Err(JiraFailure::Failed { reason })
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/jira/http_jira_issues_tests.rs"]
mod tests;
