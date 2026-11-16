use std::time::Duration;

use async_trait::async_trait;
use reqwest::header::{ACCEPT, AUTHORIZATION};
use reqwest::{Response, StatusCode};
use rekall_core::connection::{AtlassianCredential, AtlassianDeployment, AtlassianFailure, AtlassianSite, ConfluencePage, ConfluenceSpace, ConfluenceSpaces};
use serde::Deserialize;

use super::storage_format::to_markdown;
use crate::jira::{authorization, jira_client};
use crate::probes::failure_chain;

const TIMEOUT: Duration = Duration::from_secs(30);
const REST_API: &str = "/rest/api";
const CLOUD_CONTEXT: &str = "/wiki";
const CLOUD_HOST_SUFFIX: &str = ".atlassian.net";
const PAGE_BATCH: u16 = 25;
const SPACE_LISTING_SIZE: u16 = 250;

#[derive(Deserialize)]
struct SpaceAnswer {
    key: String,
    #[serde(default)]
    name: String,
    description: Option<SpaceDescription>,
    homepage: Option<Identified>,
}

#[derive(Deserialize)]
struct SpaceListing {
    #[serde(default)]
    results: Vec<SpaceAnswer>,
}

#[derive(Deserialize)]
struct SpaceDescription {
    plain: Option<PlainText>,
}

#[derive(Deserialize)]
struct PlainText {
    #[serde(default)]
    value: String,
}

#[derive(Deserialize)]
struct Identified {
    id: String,
}

#[derive(Deserialize)]
struct ContentAnswer {
    #[serde(default)]
    results: Vec<FoundPage>,
    #[serde(rename = "_links", default)]
    links: ContentLinks,
}

#[derive(Default, Deserialize)]
struct ContentLinks {
    base: Option<String>,
    next: Option<String>,
}

#[derive(Deserialize)]
struct FoundPage {
    id: String,
    #[serde(default)]
    title: String,
    body: Option<PageBody>,
    metadata: Option<PageMetadata>,
    #[serde(rename = "_links", default)]
    links: PageLinks,
}

#[derive(Deserialize)]
struct PageBody {
    storage: Option<StorageBody>,
}

#[derive(Deserialize)]
struct StorageBody {
    #[serde(default)]
    value: String,
}

#[derive(Deserialize)]
struct PageMetadata {
    labels: Option<LabelResults>,
}

#[derive(Deserialize)]
struct LabelResults {
    #[serde(default)]
    results: Vec<Label>,
}

#[derive(Deserialize)]
struct Label {
    name: String,
}

#[derive(Default, Deserialize)]
struct PageLinks {
    webui: Option<String>,
}

/// What Confluence says when it refuses a request.
#[derive(Deserialize)]
struct ConfluenceError {
    message: Option<String>,
}

/// Confluence's REST API over HTTPS with `reqwest`, through the connection's proxy when it names one.
/// Cloud's site URL is taken to end in `/wiki` even when the person left it out.
pub struct HttpConfluenceSpaces {
    timeout: Duration,
}

impl HttpConfluenceSpaces {
    pub fn new() -> Self {
        Self { timeout: TIMEOUT }
    }
}

impl Default for HttpConfluenceSpaces {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ConfluenceSpaces for HttpConfluenceSpaces {
    async fn space(&self, site: &AtlassianSite, credential: &AtlassianCredential, space_key: &str) -> Result<Option<ConfluenceSpace>, AtlassianFailure> {
        let client = jira_client(site, self.timeout).map_err(|reason| AtlassianFailure::Failed { reason })?;
        let answer = client
            .get(format!("{}/space/{space_key}", rest_root(site, credential.deployment())))
            .query(&[("expand", "description.plain,homepage")])
            .header(AUTHORIZATION, authorization(credential)?)
            .header(ACCEPT, "application/json")
            .send()
            .await;
        let response = match answer {
            Ok(response) if response.status() == StatusCode::NOT_FOUND => return Ok(None),
            answer => accepted(answer).await?,
        };
        Ok(Some(space_of(response.json::<SpaceAnswer>().await.map_err(unreadable)?)))
    }

    async fn spaces(&self, site: &AtlassianSite, credential: &AtlassianCredential, query: &str, limit: u16) -> Result<Vec<ConfluenceSpace>, AtlassianFailure> {
        let client = jira_client(site, self.timeout).map_err(|reason| AtlassianFailure::Failed { reason })?;
        let answer = client
            .get(format!("{}/space", rest_root(site, credential.deployment())))
            .query(&[("limit", SPACE_LISTING_SIZE.to_string())])
            .header(AUTHORIZATION, authorization(credential)?)
            .header(ACCEPT, "application/json")
            .send()
            .await;
        let listed = accepted(answer).await?.json::<SpaceListing>().await.map_err(unreadable)?;
        let wanted = query.trim().to_lowercase();
        let matching = listed.results.into_iter().map(space_of).filter(|space| wanted.is_empty() || space.key.to_lowercase().contains(&wanted) || space.name.to_lowercase().contains(&wanted));
        Ok(matching.take(usize::from(limit)).collect())
    }

    async fn pages(&self, site: &AtlassianSite, credential: &AtlassianCredential, space_key: &str, limit: u16) -> Result<Vec<ConfluencePage>, AtlassianFailure> {
        let client = jira_client(site, self.timeout).map_err(|reason| AtlassianFailure::Failed { reason })?;
        let root = rest_root(site, credential.deployment());
        let mut pages: Vec<ConfluencePage> = Vec::new();
        while pages.len() < usize::from(limit) {
            let batch = PAGE_BATCH.min(limit - pages.len() as u16).to_string();
            let start = pages.len().to_string();
            let answer = client
                .get(format!("{root}/content"))
                .query(&[("spaceKey", space_key), ("type", "page"), ("status", "current"), ("expand", "body.storage,metadata.labels"), ("limit", &batch), ("start", &start)])
                .header(AUTHORIZATION, authorization(credential)?)
                .header(ACCEPT, "application/json")
                .send()
                .await;
            let content = accepted(answer).await?.json::<ContentAnswer>().await.map_err(unreadable)?;
            let base = content.links.base.clone().unwrap_or_else(|| root.clone());
            let has_more = content.links.next.is_some();
            pages.extend(content.results.into_iter().map(|found| page_of(found, &base)));
            if !has_more {
                break;
            }
        }
        Ok(pages)
    }
}

pub(crate) fn rest_root(site: &AtlassianSite, deployment: AtlassianDeployment) -> String {
    let url = site.url.trim_end_matches('/');
    let is_cloud_without_context = deployment == AtlassianDeployment::Cloud && url.contains(CLOUD_HOST_SUFFIX) && !url.ends_with(CLOUD_CONTEXT);
    if is_cloud_without_context {
        format!("{url}{CLOUD_CONTEXT}{REST_API}")
    } else {
        format!("{url}{REST_API}")
    }
}

fn space_of(found: SpaceAnswer) -> ConfluenceSpace {
    let description = found.description.and_then(|description| description.plain).map(|plain| plain.value.trim().to_string()).filter(|text| !text.is_empty());
    ConfluenceSpace { key: found.key, name: found.name, description, home_page_id: found.homepage.map(|page| page.id) }
}

fn page_of(found: FoundPage, base: &str) -> ConfluencePage {
    let storage = found.body.and_then(|body| body.storage).map(|storage| storage.value).unwrap_or_default();
    let labels = found.metadata.and_then(|metadata| metadata.labels).map(|labels| labels.results.into_iter().map(|label| label.name).collect()).unwrap_or_default();
    let url = found.links.webui.map_or_else(String::new, |webui| format!("{}{webui}", base.trim_end_matches('/')));
    ConfluencePage { id: found.id, title: found.title, labels, markdown: to_markdown(&storage), url }
}

fn unreadable(failure: reqwest::Error) -> AtlassianFailure {
    AtlassianFailure::Failed { reason: format!("its answer did not read as Confluence content ({failure})") }
}

/// The response when Confluence answered, or why it did not, in Confluence's own words when it gave some.
async fn accepted(answer: reqwest::Result<Response>) -> Result<Response, AtlassianFailure> {
    let response = answer.map_err(|failure| AtlassianFailure::Failed { reason: failure_chain(&failure) })?;
    match response.status() {
        status if status.is_success() => Ok(response),
        status @ (StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN) => Err(AtlassianFailure::Rejected { status: status.as_u16() }),
        status => {
            let message = response.json::<ConfluenceError>().await.ok().and_then(|error| error.message).unwrap_or_default();
            let reason = if message.is_empty() { format!("it answered HTTP {}", status.as_u16()) } else { format!("it answered HTTP {}: {message}", status.as_u16()) };
            Err(AtlassianFailure::Failed { reason })
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/confluence/http_confluence_spaces_tests.rs"]
mod tests;
