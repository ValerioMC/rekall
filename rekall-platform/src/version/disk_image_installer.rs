//! `BundleInstaller` over the macOS tools: `hdiutil` mounts the image, `ditto` copies the bundle.
//! Files written this way carry no quarantine attribute.

use std::path::Path;

use async_trait::async_trait;
use tokio::process::Command;

use super::{BundleError, BundleInstaller};

const ATTACH_FAILED: &str = "the disk image could not be opened";
const DETACH_FAILED: &str = "the disk image could not be ejected";
const COPY_FAILED: &str = "the new version could not be copied";

pub struct DiskImageInstaller;

#[async_trait]
impl BundleInstaller for DiskImageInstaller {
    async fn attach(&self, image: &Path, mount_point: &Path) -> Result<(), BundleError> {
        let mut attach = Command::new("hdiutil");
        attach.args(["attach", "-nobrowse", "-readonly", "-noautoopen", "-mountpoint"]).arg(mount_point).arg(image);
        run(&mut attach, ATTACH_FAILED).await
    }

    /// A busy volume is forced out: the update has already copied what it needed.
    async fn detach(&self, mount_point: &Path) -> Result<(), BundleError> {
        let mut quiet = Command::new("hdiutil");
        quiet.args(["detach", "-quiet"]).arg(mount_point);
        if run(&mut quiet, DETACH_FAILED).await.is_ok() {
            return Ok(());
        }
        let mut forced = Command::new("hdiutil");
        forced.args(["detach", "-force", "-quiet"]).arg(mount_point);
        run(&mut forced, DETACH_FAILED).await
    }

    async fn copy_bundle(&self, source: &Path, target: &Path) -> Result<(), BundleError> {
        let mut copy = Command::new("ditto");
        copy.arg(source).arg(target);
        run(&mut copy, COPY_FAILED).await
    }
}

async fn run(command: &mut Command, action: &str) -> Result<(), BundleError> {
    let output = command.output().await.map_err(|failure| BundleError::new(action, failure))?;
    if output.status.success() {
        return Ok(());
    }
    Err(BundleError::new(action, String::from_utf8_lossy(&output.stderr).trim()))
}
