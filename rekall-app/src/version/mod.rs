//! The version this build is and whether a newer release exists: `GET /api/version`.
//! The running version is stamped by the release workflow (`REKALL_VERSION`); the newest one is
//! read from GitHub's latest release, so the tag that triggered a build is what gets compared.
//! On macOS `update_installer` puts that release in place of the running Rekall.app.

mod bundle_files;
mod bundle_installer;
mod disk_image_installer;
mod release_feed;
mod release_version;
mod update_checker;
mod update_installer;
mod version_controller;
mod version_status;

pub use bundle_installer::{BundleError, BundleInstaller};
pub use disk_image_installer::DiskImageInstaller;
pub use release_feed::{
    GithubReleaseFeed, Release, ReleaseAsset, ReleaseDownloadError, ReleaseDownloader, ReleaseFeed, ReleaseFeedError,
    DEFAULT_RELEASE_URL,
};
pub use release_version::{InvalidVersion, ReleaseVersion, RUNNING_VERSION};
pub use update_checker::UpdateChecker;
pub use update_installer::{app_bundle_of, install, UpdateInstallError, UpdateTarget, DOWNLOAD_PREFIX};
pub use version_controller::routes;
pub use version_status::{LatestRelease, UpdateCheck, VersionStatus};
