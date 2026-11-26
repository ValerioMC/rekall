//! A picture sent to a task manager: GitHub's attachment endpoint (the one `gh` uses, which GitHub
//! does not document), GitLab's project uploads and Jira's issue attachments.

use std::time::Duration;

use reqwest::header::{ACCEPT, CONTENT_TYPE, USER_AGENT};
use reqwest::multipart::{Form, Part};
use reqwest::{Method, RequestBuilder, Response, StatusCode};
use rekall_core::connection::{github_api_root, github_upload_root, AtlassianDeployment, TrackerEndpoint, TrackerFailure};
use rekall_core::picture::PicturePath;
use rekall_store::ConnectionKind;
use serde::de::DeserializeOwned;
use serde::Deserialize;

use crate::probes::{client_through, failure_chain};

use super::http_task_tracker::{checked, gitlab_project, jira_api, jira_deployment, not_a_task_manager, signature_of};

/// Longer than a call that carries text: a picture is up to 10 MB.
const UPLOAD_TIMEOUT: Duration = Duration::from_secs(120);
const GITHUB_ACCEPT: &str = "application/vnd.github+json";

#[derive(Deserialize)]
struct GithubRepository {
    id: u64,
}

#[derive(Deserialize)]
struct UploadedAsset {
    url: String,
}

#[derive(Deserialize)]
struct JiraAttachment {
    filename: String,
    content: String,
}

/// One picture on its way to a task manager.
pub(super) struct PictureUpload<'a> {
    pub endpoint: &'a TrackerEndpoint,
    pub picture: &'a PicturePath,
    pub bytes: Vec<u8>,
}

impl PictureUpload<'_> {
    pub(super) async fn send(self, issue_key: Option<&str>, github_uploads_override: Option<&str>) -> Result<String, TrackerFailure> {
        match self.endpoint.kind {
            ConnectionKind::Github => self.upload_to_github(github_uploads_override).await,
            ConnectionKind::Gitlab => self.upload_to_gitlab().await,
            ConnectionKind::Jira => self.attach_to_jira(issue_key).await,
            ConnectionKind::Kubernetes | ConnectionKind::Grafana | ConnectionKind::Confluence => Err(not_a_task_manager()),
        }
    }

    /// GitHub takes the bytes against the repository's numeric id and answers the asset's address.
    async fn upload_to_github(self, uploads_override: Option<&str>) -> Result<String, TrackerFailure> {
        let site = self.endpoint.site_url.trim_end_matches('/');
        let uploads = uploads_override
            .or_else(|| github_upload_root(site))
            .ok_or_else(|| failed("attaching pictures is not supported on GitHub Enterprise Server"))?;
        let lookup = self.request(Method::GET, format!("{}/repos/{}", github_api_root(site), self.endpoint.location))?;
        let repository: GithubRepository = read(answer_to(lookup.header(ACCEPT, GITHUB_ACCEPT)).await?).await?;
        let query = [("name", self.picture.file_name()), ("content_type", self.picture.format().content_type()), ("repository_id", &repository.id.to_string())];
        let upload = self
            .request(Method::POST, format!("{uploads}/user-attachments/assets"))?
            .query(&query)
            .header(ACCEPT, GITHUB_ACCEPT)
            .header(CONTENT_TYPE, "application/octet-stream")
            .body(self.bytes);
        let response = upload.send().await.map_err(|failure| failed(&failure_chain(&failure)))?;
        if response.status() == StatusCode::NOT_FOUND {
            return Err(failed("it answered HTTP 404: the token may not write to the repository (attachments take a classic or OAuth token)"));
        }
        read::<UploadedAsset>(checked(response).await?).await.map(|asset| asset.url)
    }

    /// GitLab answers an address that its markdown resolves inside the same project.
    async fn upload_to_gitlab(self) -> Result<String, TrackerFailure> {
        let site = self.endpoint.site_url.trim_end_matches('/');
        let url = format!("{site}/api/v4/projects/{}/uploads", gitlab_project(&self.endpoint.location));
        let upload = self.request(Method::POST, url)?.multipart(self.form()?);
        read::<UploadedAsset>(answer_to(upload).await?).await.map(|upload| upload.url)
    }

    /// Cloud is referred to by the attachment's content address, Data Center by its file name.
    async fn attach_to_jira(self, issue_key: Option<&str>) -> Result<String, TrackerFailure> {
        let deployment = jira_deployment(self.endpoint)?;
        let key = issue_key.ok_or_else(|| failed("an attachment belongs to an issue, and there is none"))?;
        let site = self.endpoint.site_url.trim_end_matches('/');
        let url = format!("{site}{}/issue/{key}/attachments", jira_api(deployment));
        let upload = self.request(Method::POST, url)?.header("X-Atlassian-Token", "no-check").multipart(self.form()?);
        let attachments: Vec<JiraAttachment> = read(answer_to(upload).await?).await?;
        let attachment = attachments.into_iter().next().ok_or_else(|| failed("its answer named no attachment"))?;
        Ok(match deployment {
            AtlassianDeployment::Cloud => attachment.content,
            AtlassianDeployment::DataCenter => attachment.filename,
        })
    }

    fn request(&self, method: Method, url: String) -> Result<RequestBuilder, TrackerFailure> {
        let client = client_through(self.endpoint.proxy_url.as_deref(), UPLOAD_TIMEOUT).map_err(|reason| TrackerFailure::Failed { reason })?;
        let (name, value) = signature_of(self.endpoint)?;
        Ok(client.request(method, url).header(name, value).header(USER_AGENT, "rekall"))
    }

    fn form(&self) -> Result<Form, TrackerFailure> {
        let part = Part::bytes(self.bytes.clone())
            .file_name(self.picture.file_name().to_string())
            .mime_str(self.picture.format().content_type())
            .map_err(|failure| failed(&failure_chain(&failure)))?;
        Ok(Form::new().part("file", part))
    }
}

async fn answer_to(request: RequestBuilder) -> Result<Response, TrackerFailure> {
    checked(request.send().await.map_err(|failure| failed(&failure_chain(&failure)))?).await
}

async fn read<T: DeserializeOwned>(response: Response) -> Result<T, TrackerFailure> {
    response.json::<T>().await.map_err(|failure| failed(&format!("its answer did not read as an upload ({failure})")))
}

fn failed(reason: &str) -> TrackerFailure {
    TrackerFailure::Failed { reason: reason.to_string() }
}

#[cfg(test)]
#[path = "../../tests/unit/tracker/http_picture_upload_tests.rs"]
mod tests;
