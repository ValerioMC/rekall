use std::sync::Mutex;

use async_trait::async_trait;

use super::*;
use crate::version::ReleaseAsset;

const DMG_URL: &str = "https://github.com/ValerioMC/rekall/releases/download/v0.2.0/Rekall-macos-arm64.dmg";

/// Serves one release; `download` writes the URL into the file so a test sees what was fetched.
struct FakeRelease {
    release: Option<Release>,
    downloads: Mutex<Vec<String>>,
}

impl FakeRelease {
    fn new(tag: &str, url: &str) -> Self {
        let release = Release {
            tag: tag.to_string(),
            page_url: format!("https://github.com/ValerioMC/rekall/releases/tag/{tag}"),
            assets: vec![
                ReleaseAsset { name: "rekall-v0.2.0-linux-x64".into(), download_url: format!("{DOWNLOAD_PREFIX}linux") },
                ReleaseAsset { name: "Rekall-macos-arm64.dmg".into(), download_url: url.to_string() },
            ],
        };
        Self { release: Some(release), downloads: Mutex::new(Vec::new()) }
    }

    fn downloaded(&self) -> Vec<String> {
        self.downloads.lock().unwrap().clone()
    }
}

#[async_trait]
impl ReleaseFeed for FakeRelease {
    async fn latest(&self) -> Result<Option<Release>, ReleaseFeedError> {
        Ok(self.release.clone())
    }
}

#[async_trait]
impl ReleaseDownloader for FakeRelease {
    async fn download(&self, url: &str, target: &Path) -> Result<(), ReleaseDownloadError> {
        self.downloads.lock().unwrap().push(url.to_string());
        std::fs::write(target, url).map_err(|failure| ReleaseDownloadError::Unreachable(failure.to_string()))
    }
}

/// "Mounts" an image by creating `Rekall.app` with a version marker inside the mount point.
#[derive(Default)]
struct FakeInstaller {
    copy_fails: bool,
    detached: Mutex<bool>,
}

impl FakeInstaller {
    fn was_detached(&self) -> bool {
        *self.detached.lock().unwrap()
    }
}

#[async_trait]
impl BundleInstaller for FakeInstaller {
    async fn attach(&self, image: &Path, mount_point: &Path) -> Result<(), BundleError> {
        assert!(image.is_file());
        let app = mount_point.join("Rekall.app");
        std::fs::create_dir_all(&app).unwrap();
        std::fs::write(app.join("version"), "new").unwrap();
        Ok(())
    }

    async fn detach(&self, _mount_point: &Path) -> Result<(), BundleError> {
        *self.detached.lock().unwrap() = true;
        Ok(())
    }

    async fn copy_bundle(&self, source: &Path, target: &Path) -> Result<(), BundleError> {
        if self.copy_fails {
            return Err(BundleError::new("copy", "disk full"));
        }
        std::fs::create_dir_all(target).unwrap();
        std::fs::copy(source.join("version"), target.join("version")).unwrap();
        Ok(())
    }
}

struct Scratch {
    _root: tempfile::TempDir,
    bundle: PathBuf,
    executable: PathBuf,
    work_dir: PathBuf,
}

impl Scratch {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let bundle = root.path().join("Applications").join("Rekall.app");
        let executable = bundle.join("Contents").join("MacOS").join("rekall-desktop");
        std::fs::create_dir_all(executable.parent().unwrap()).unwrap();
        std::fs::write(bundle.join("version"), "old").unwrap();
        Self { work_dir: root.path().join("work"), _root: root, bundle, executable }
    }

    fn target<'a>(&'a self, release: &'a FakeRelease, installer: &'a FakeInstaller) -> UpdateTarget<'a> {
        UpdateTarget {
            feed: release,
            downloader: release,
            installer,
            running_version: "0.1.0",
            running_executable: &self.executable,
            work_dir: &self.work_dir,
        }
    }

    fn installed_version(&self) -> String {
        std::fs::read_to_string(self.bundle.join("version")).unwrap()
    }

    fn applications(&self) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(self.bundle.parent().unwrap())
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }
}

#[tokio::test]
async fn the_new_bundle_replaces_the_running_one() {
    let scratch = Scratch::new();
    let release = FakeRelease::new("v0.2.0", DMG_URL);
    let installer = FakeInstaller::default();

    let version = install(scratch.target(&release, &installer)).await.unwrap();

    assert_eq!(version, "0.2.0");
    assert_eq!(scratch.installed_version(), "new");
    assert_eq!(release.downloaded(), vec![DMG_URL.to_string()]);
    assert!(installer.was_detached());
    assert_eq!(scratch.applications(), vec!["Rekall.app"]);
    assert!(!scratch.work_dir.join(IMAGE_NAME).exists());
}

#[tokio::test]
async fn a_failed_copy_leaves_the_running_bundle_untouched() {
    let scratch = Scratch::new();
    let release = FakeRelease::new("v0.2.0", DMG_URL);
    let installer = FakeInstaller { copy_fails: true, ..FakeInstaller::default() };

    let failure = install(scratch.target(&release, &installer)).await.unwrap_err();

    assert!(matches!(failure, UpdateInstallError::Bundle(_)), "{failure}");
    assert_eq!(scratch.installed_version(), "old");
    assert!(installer.was_detached());
    assert_eq!(scratch.applications(), vec!["Rekall.app"]);
}

#[tokio::test]
async fn nothing_is_downloaded_when_already_up_to_date() {
    let scratch = Scratch::new();
    let release = FakeRelease::new("v0.1.0", DMG_URL);
    let installer = FakeInstaller::default();

    let failure = install(scratch.target(&release, &installer)).await.unwrap_err();

    assert!(matches!(failure, UpdateInstallError::AlreadyCurrent(_)), "{failure}");
    assert!(release.downloaded().is_empty());
}

#[tokio::test]
async fn a_disk_image_outside_rekalls_releases_is_refused() {
    let scratch = Scratch::new();
    let release = FakeRelease::new("v0.2.0", "https://example.test/Rekall-macos-arm64.dmg");
    let installer = FakeInstaller::default();

    let failure = install(scratch.target(&release, &installer)).await.unwrap_err();

    assert!(matches!(failure, UpdateInstallError::NoDiskImage(_)), "{failure}");
    assert!(release.downloaded().is_empty());
    assert_eq!(scratch.installed_version(), "old");
}

#[tokio::test]
async fn no_published_release_is_refused() {
    let scratch = Scratch::new();
    let release = FakeRelease { release: None, downloads: Mutex::new(Vec::new()) };
    let installer = FakeInstaller::default();

    let failure = install(scratch.target(&release, &installer)).await.unwrap_err();

    assert!(matches!(failure, UpdateInstallError::NoRelease), "{failure}");
}

#[tokio::test]
async fn a_development_binary_is_refused_before_anything_is_asked() {
    let scratch = Scratch::new();
    let release = FakeRelease::new("v0.2.0", DMG_URL);
    let installer = FakeInstaller::default();
    let executable = scratch.work_dir.join("target/debug/rekall-desktop");
    let target = UpdateTarget { running_executable: &executable, ..scratch.target(&release, &installer) };

    let failure = install(target).await.unwrap_err();

    assert!(matches!(failure, UpdateInstallError::NotInBundle), "{failure}");
    assert!(release.downloaded().is_empty());
}

#[test]
fn the_bundle_is_found_above_the_executable() {
    let executable = Path::new("/Applications/Rekall.app/Contents/MacOS/rekall-desktop");

    assert_eq!(app_bundle_of(executable).unwrap(), PathBuf::from("/Applications/Rekall.app"));
}

#[test]
fn the_staging_and_previous_bundles_are_hidden_siblings() {
    let bundle = Path::new("/Applications/Rekall.app");

    assert_eq!(sibling(bundle, "new"), PathBuf::from("/Applications/.Rekall.app.new"));
}
