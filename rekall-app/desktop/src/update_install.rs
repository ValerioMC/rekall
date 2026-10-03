//! The `installUpdate()` bridge: installs the newest release over this Rekall.app and restarts into
//! it. The app downloads the disk image itself, so the new version carries no quarantine flag and
//! opens without `xattr`. macOS only; elsewhere the console keeps its download link.

use std::sync::Arc;

use tauri::{AppHandle, Manager, Runtime};
use tracing::{error, info};

use crate::exit_guard::{self, Leaving};
use crate::Shell;

const WORK_FOLDER: &str = "rekall-update";
const WAIT_THEN_OPEN: &str =
    r#"while kill -0 "$1" 2>/dev/null; do sleep 0.2; done; exec /usr/bin/open "$2""#;

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
    relaunch_in_foreground(&app)?;
    Ok(true)
}

/// Waits for this process to end, then opens the bundle through LaunchServices, which brings the
/// new app to the front; a restart that execs the binary directly opens behind other windows.
fn relaunch_in_foreground<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let executable = std::env::current_exe().map_err(|failure| failure.to_string())?;
    let bundle =
        rekall_app::version::app_bundle_of(&executable).map_err(|failure| failure.to_string())?;
    std::process::Command::new("/bin/sh")
        .args(["-c", WAIT_THEN_OPEN, "sh", &std::process::id().to_string()])
        .arg(&bundle)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|failure| format!("Could not relaunch Rekall: {failure}"))?;
    app.exit(0);
    Ok(())
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
