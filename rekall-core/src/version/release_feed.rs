use async_trait::async_trait;

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
