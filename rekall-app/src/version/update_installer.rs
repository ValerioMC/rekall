//! Installs the newest release over the running Rekall.app. The app downloads the disk image
//! itself, so no quarantine attribute is set and the new version opens without `xattr`. The old
//! bundle is kept aside until the new one is in place, and put back if the swap fails.

use std::path::{Path, PathBuf};

use tracing::{info, warn};

use super::{
    bundle_files, BundleError, BundleInstaller, InvalidVersion, Release, ReleaseDownloadError, ReleaseDownloader,
    ReleaseFeed, ReleaseFeedError, ReleaseVersion,
};

/// Only files of this repository's releases are ever installed.
pub const DOWNLOAD_PREFIX: &str = "https://github.com/ValerioMC/rekall/releases/download/";
const DISK_IMAGE_SUFFIX: &str = ".dmg";
const IMAGE_NAME: &str = "Rekall-update.dmg";
const MOUNT_NAME: &str = "volume";

#[derive(Debug, thiserror::Error)]
pub enum UpdateInstallError {
    #[error("Rekall is not running from an installed app, so it has to be updated by hand")]
    NotInBundle,
    #[error("no release has been published yet")]
    NoRelease,
    #[error("Rekall {0} is already the newest version")]
    AlreadyCurrent(String),
    #[error(transparent)]
    Unorderable(#[from] InvalidVersion),
    #[error("release {0} carries no disk image from Rekall's own releases")]
    NoDiskImage(String),
    #[error(transparent)]
    Feed(#[from] ReleaseFeedError),
    #[error(transparent)]
    Download(#[from] ReleaseDownloadError),
    #[error(transparent)]
    Bundle(#[from] BundleError),
}

/// What the update acts on: the tools, this build and where it may write scratch files.
pub struct UpdateTarget<'a> {
    pub feed: &'a dyn ReleaseFeed,
    pub downloader: &'a dyn ReleaseDownloader,
    pub installer: &'a dyn BundleInstaller,
    pub running_version: &'a str,
    pub running_executable: &'a Path,
    pub work_dir: &'a Path,
}

/// Returns the installed version. Nothing is downloaded unless a newer disk image exists.
pub async fn install(target: UpdateTarget<'_>) -> Result<String, UpdateInstallError> {
    let bundle = app_bundle_of(target.running_executable)?;
    let release = target.feed.latest().await?.ok_or(UpdateInstallError::NoRelease)?;
    let latest = newer_version(&release, target.running_version)?;
    let url = disk_image_url(&release).ok_or_else(|| UpdateInstallError::NoDiskImage(latest.clone()))?;

    bundle_files::reset_folder(target.work_dir).await?;
    let image = target.work_dir.join(IMAGE_NAME);
    target.downloader.download(url, &image).await?;
    let mount_point = target.work_dir.join(MOUNT_NAME);
    bundle_files::reset_folder(&mount_point).await?;
    target.installer.attach(&image, &mount_point).await?;
    let replaced = match bundle_files::find_app_in(&mount_point).await {
        Ok(source) => replace_bundle(target.installer, &source, &bundle).await,
        Err(failure) => Err(failure),
    };
    if let Err(failure) = target.installer.detach(&mount_point).await {
        warn!("The update's disk image was left mounted: {failure}");
    }
    if let Err(failure) = bundle_files::remove_tree(&image).await {
        warn!("The update's disk image was not deleted: {failure}");
    }
    replaced?;
    info!("Installed Rekall {latest} over {}", bundle.display());
    Ok(latest)
}

/// The `.app` directory the executable runs from; a development binary has none.
pub fn app_bundle_of(executable: &Path) -> Result<PathBuf, UpdateInstallError> {
    executable
        .ancestors()
        .find(|path| path.extension().is_some_and(|extension| extension == "app"))
        .map(Path::to_path_buf)
        .ok_or(UpdateInstallError::NotInBundle)
}

/// The release's version without its `v`, when it is newer than `running`.
fn newer_version(release: &Release, running: &str) -> Result<String, UpdateInstallError> {
    let current = running.parse::<ReleaseVersion>()?;
    let latest = release.tag.parse::<ReleaseVersion>()?;
    if latest <= current {
        return Err(UpdateInstallError::AlreadyCurrent(running.to_string()));
    }
    Ok(release.tag.trim_start_matches('v').to_string())
}

fn disk_image_url(release: &Release) -> Option<&str> {
    release
        .assets
        .iter()
        .find(|asset| asset.name.ends_with(DISK_IMAGE_SUFFIX) && asset.download_url.starts_with(DOWNLOAD_PREFIX))
        .map(|asset| asset.download_url.as_str())
}

/// Copies the new bundle beside the old, then swaps names. A failed swap puts the old one back.
async fn replace_bundle(installer: &dyn BundleInstaller, source: &Path, bundle: &Path) -> Result<(), BundleError> {
    let staging = sibling(bundle, "new");
    let previous = sibling(bundle, "old");
    bundle_files::remove_tree(&staging).await?;
    if let Err(failure) = installer.copy_bundle(source, &staging).await {
        bundle_files::remove_tree(&staging).await?;
        return Err(failure);
    }
    bundle_files::remove_tree(&previous).await?;
    bundle_files::rename(bundle, &previous).await?;
    if let Err(failure) = bundle_files::rename(&staging, bundle).await {
        bundle_files::rename(&previous, bundle).await?;
        return Err(failure);
    }
    if let Err(failure) = bundle_files::remove_tree(&previous).await {
        warn!("The previous version was not deleted: {failure}");
    }
    Ok(())
}

/// `/Applications/Rekall.app` → `/Applications/.Rekall.app.new`, hidden in Finder.
fn sibling(bundle: &Path, suffix: &str) -> PathBuf {
    let name = bundle.file_name().unwrap_or_default().to_string_lossy();
    bundle.with_file_name(format!(".{name}.{suffix}"))
}

#[cfg(test)]
#[path = "../../tests/unit/version/update_installer_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "../../tests/unit/version/update_installer_live_tests.rs"]
mod live_tests;
