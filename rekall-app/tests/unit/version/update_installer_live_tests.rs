//! Installs the published release over a stand-in bundle in a temp folder, with the real GitHub
//! download, `hdiutil` and `ditto`. Mac and network only: `cargo test -p rekall-app live_ -- --ignored`.

use std::process::Command;

use super::*;
use crate::version::{DiskImageInstaller, GithubReleaseFeed, DEFAULT_RELEASE_URL};

#[tokio::test]
#[ignore]
async fn live_the_published_release_installs_without_quarantine() {
    let root = tempfile::tempdir().unwrap();
    let bundle = root.path().join("Applications").join("Rekall.app");
    let executable = bundle.join("Contents").join("MacOS").join("rekall-desktop");
    std::fs::create_dir_all(executable.parent().unwrap()).unwrap();
    let feed = GithubReleaseFeed::new(DEFAULT_RELEASE_URL);

    let version = install(UpdateTarget {
        feed: &feed,
        downloader: &feed,
        installer: &DiskImageInstaller,
        running_version: "0.0.1",
        running_executable: &executable,
        work_dir: &root.path().join("work"),
    })
    .await
    .unwrap();

    assert!(bundle.join("Contents").join("Info.plist").is_file(), "{version}");
    let attributes = Command::new("xattr").arg("-r").arg(&bundle).output().unwrap();
    let listed = String::from_utf8_lossy(&attributes.stdout);
    assert!(!listed.contains("com.apple.quarantine"), "{listed}");
}
