//! The `installUpdate()` bridge: installs the newest release over this Rekall.app and restarts into
//! it. The app downloads the disk image itself, so the new version carries no quarantine flag and
//! opens without `xattr`. macOS only; elsewhere the console keeps its download link.

use std::sync::Arc;

use tauri::{AppHandle, Manager, Runtime};
use tracing::{error, info};

use crate::exit_guard::{self, Leaving};
use crate::Shell;

const WORK_FOLDER: &str = "rekall-update";

/// Resolves false when the user kept a live Claude session instead, true once the restart is
/// requested. A refusal rejects with the reason, so the console can offer the browser download.
#[tauri::command]
pub async fn install_update<R: Runtime>(app: AppHandle<R>) -> Result<bool, String> {
    if !cfg!(target_os = "macos") {
        return Err("Installing from the app is only available on macOS".into());
    }
    let owns_server = app.state::<Arc<Shell>>().running.lock().await.is_some();
    if owns_server && !exit_guard::may_leave(&app, crate::port(), Leaving::Update).await {
        return Ok(false);
    }
    let version = install().await.map_err(|failure| {
        error!("The update was not installed: {failure}");
        failure
    })?;
    info!("Restarting into Rekall {version}");
    app.request_restart();
    Ok(true)
}

async fn install() -> Result<String, String> {
    use rekall_app::version::{self, DiskImageInstaller, GithubReleaseFeed, UpdateTarget};

    let feed = GithubReleaseFeed::new(&crate::server_config().update_check.url);
    let running_executable = std::env::current_exe().map_err(|failure| failure.to_string())?;
    let work_dir = std::env::temp_dir().join(WORK_FOLDER);
    version::install(UpdateTarget {
        feed: &feed,
        downloader: &feed,
        installer: &DiskImageInstaller,
        running_version: version::RUNNING_VERSION,
        running_executable: &running_executable,
        work_dir: &work_dir,
    })
    .await
    .map_err(|failure| failure.to_string())
}
