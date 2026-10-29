//! The newest release, read from GitHub's latest release: `GithubReleaseFeed` implements the
//! `ReleaseFeed` port of `rekall_core::version`. On macOS `update_installer` puts that release in
//! place of the running Rekall.app.

mod bundle_files;
mod bundle_installer;
mod disk_image_installer;
mod release_feed;
mod update_installer;

pub use bundle_installer::{BundleError, BundleInstaller};
pub use disk_image_installer::DiskImageInstaller;
pub use release_feed::{GithubReleaseFeed, ReleaseDownloadError, ReleaseDownloader, DEFAULT_RELEASE_URL};
pub use rekall_core::version::RUNNING_VERSION;
pub use update_installer::{app_bundle_of, install, UpdateInstallError, UpdateTarget, DOWNLOAD_PREFIX};
