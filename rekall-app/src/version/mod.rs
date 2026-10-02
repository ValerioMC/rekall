//! The version this build is and whether a newer release exists: `GET /api/version`.
//! The running version is stamped by the release workflow (`REKALL_VERSION`); the newest one is
//! read from GitHub's latest release, so the tag that triggered a build is what gets compared.

mod release_feed;
mod release_version;
mod update_checker;
mod version_controller;
mod version_status;

pub use release_feed::{GithubReleaseFeed, Release, ReleaseAsset, ReleaseFeed, ReleaseFeedError, DEFAULT_RELEASE_URL};
pub use release_version::{InvalidVersion, ReleaseVersion, RUNNING_VERSION};
pub use update_checker::UpdateChecker;
pub use version_controller::routes;
pub use version_status::{LatestRelease, UpdateCheck, VersionStatus};
