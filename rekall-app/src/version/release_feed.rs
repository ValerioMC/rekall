use std::time::Duration;

use async_trait::async_trait;
use serde::Deserialize;

pub const DEFAULT_RELEASE_URL: &str = "https://api.github.com/repos/ValerioMC/rekall/releases/latest";

const HTTP_TIMEOUT: Duration = Duration::from_secs(8);

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

#[async_trait]
pub trait ReleaseFeed: Send + Sync {
    /// The newest published release, or `None` when nothing has been published yet.
    async fn latest(&self) -> Result<Option<Release>, ReleaseFeedError>;
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
