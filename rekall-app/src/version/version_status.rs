use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UpdateCheck {
    UpToDate,
    UpdateAvailable,
    /// The release feed could not be read, or one of the two versions could not be ordered.
    Unavailable,
    /// `rekall.update-check.enabled` is off: nothing was asked of the network.
    Disabled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LatestRelease {
    pub version: String,
    pub release_url: String,
    /// The asset for the platform this build runs on, when the release carries one.
    pub download_url: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionStatus {
    pub current: String,
    pub check: UpdateCheck,
    pub latest: Option<LatestRelease>,
}
