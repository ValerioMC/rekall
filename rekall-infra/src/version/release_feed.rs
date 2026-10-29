use std::path::Path;
use std::time::Duration;

use async_trait::async_trait;
use serde::Deserialize;
use tokio::io::AsyncWriteExt;

pub const DEFAULT_RELEASE_URL: &str = "https://api.github.com/repos/ValerioMC/rekall/releases/latest";

const HTTP_TIMEOUT: Duration = Duration::from_secs(8);
/// A disk image is tens of megabytes: generous for a slow line, still bounded.
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(600);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReleaseAsset {
    pub name: String,
    pub download_url: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Release {
    pub tag: String,
    pub page_url: String,
    pub assets: Vec<ReleaseAsset>,
}

#[derive(Debug, thiserror::Error)]
pub enum ReleaseFeedError {
    #[error("the release feed could not be reached: {0}")]
    Unreachable(String),
    #[error("the release feed answered {0}")]
    Refused(u16),
    #[error("the release feed answered something that is not a release: {0}")]
    Malformed(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ReleaseDownloadError {
    #[error("the download could not be reached: {0}")]
    Unreachable(String),
    #[error("the download answered {0}")]
    Refused(u16),
    #[error("the download could not be saved to {path}: {cause}")]
    Unwritable { path: String, cause: String },
}

#[async_trait]
pub trait ReleaseFeed: Send + Sync {
    /// The newest published release, or `None` when nothing has been published yet.
    async fn latest(&self) -> Result<Option<Release>, ReleaseFeedError>;
}

/// Fetches a release's asset into a local file.
#[async_trait]
pub trait ReleaseDownloader: Send + Sync {
    /// Writes the file at `url` to `target`, replacing it.
    async fn download(&self, url: &str, target: &Path) -> Result<(), ReleaseDownloadError>;
}

pub struct GithubReleaseFeed {
    http: reqwest::Client,
    url: String,
}

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
    html_url: String,
    #[serde(default)]
    assets: Vec<GithubAsset>,
}

#[derive(Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

impl GithubReleaseFeed {
    pub fn new(url: &str) -> Self {
        let http = reqwest::Client::builder()
            .timeout(HTTP_TIMEOUT)
            .user_agent(concat!("rekall/", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("an HTTP client");
        Self { http, url: url.to_string() }
    }
}

#[async_trait]
impl ReleaseFeed for GithubReleaseFeed {
    async fn latest(&self) -> Result<Option<Release>, ReleaseFeedError> {
        let response = self
            .http
            .get(&self.url)
            .header("Accept", "application/vnd.github+json")
            .send()
            .await
            .map_err(|failure| ReleaseFeedError::Unreachable(failure.to_string()))?;
        match response.status().as_u16() {
            200 => {}
            404 => return Ok(None),
            other => return Err(ReleaseFeedError::Refused(other)),
        }
        let release: GithubRelease =
            response.json().await.map_err(|failure| ReleaseFeedError::Malformed(failure.to_string()))?;
        Ok(Some(Release {
            tag: release.tag_name,
            page_url: release.html_url,
            assets: release
                .assets
                .into_iter()
                .map(|asset| ReleaseAsset { name: asset.name, download_url: asset.browser_download_url })
                .collect(),
        }))
    }
}

#[async_trait]
impl ReleaseDownloader for GithubReleaseFeed {
    async fn download(&self, url: &str, target: &Path) -> Result<(), ReleaseDownloadError> {
        let unreachable = |failure: reqwest::Error| ReleaseDownloadError::Unreachable(failure.to_string());
        let unwritable =
            |failure: std::io::Error| ReleaseDownloadError::Unwritable { path: target.display().to_string(), cause: failure.to_string() };
        let mut response = self.http.get(url).timeout(DOWNLOAD_TIMEOUT).send().await.map_err(unreachable)?;
        if !response.status().is_success() {
            return Err(ReleaseDownloadError::Refused(response.status().as_u16()));
        }
        let mut file = tokio::fs::File::create(target).await.map_err(unwritable)?;
        while let Some(chunk) = response.chunk().await.map_err(unreachable)? {
            file.write_all(&chunk).await.map_err(unwritable)?;
        }
        file.flush().await.map_err(unwritable)
    }
}
