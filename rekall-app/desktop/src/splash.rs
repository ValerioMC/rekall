//! The splash screen: what it says while the server comes up, and the hand-over to the console
//! once it answers.

use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tauri::{AppHandle, LogicalSize, Manager, WebviewWindow};
use tracing::error;

use crate::bridges;
use crate::shell::{base, log_file, Shell};

/// The splash stays at least this long, so it is seen even when the server answers at once.
const SPLASH_MINIMUM: Duration = Duration::from_millis(1500);
/// How long the console gets to load before the window fades in regardless.
const CONSOLE_LOAD_LIMIT: Duration = Duration::from_secs(5);
/// What the splash window is when the console takes over: the size it is restored to, and the
/// smallest it can be made.
const CONSOLE_SIZE: LogicalSize<f64> = LogicalSize::new(1440.0, 900.0);
const CONSOLE_MINIMUM_SIZE: LogicalSize<f64> = LogicalSize::new(960.0, 600.0);

pub(crate) fn status(handle: &AppHandle, text: &str) {
    let shell = handle.state::<Arc<Shell>>();
    *shell.status.lock().expect("never poisoned") = Some(text.to_string());
    if let Some(window) = handle.get_webview_window("main") {
        replay_splash(&window);
    }
}

pub(crate) fn fail(handle: &AppHandle, title: &str, detail: &str) {
    error!(%title, %detail, "Rekall could not start");
    let shell = handle.state::<Arc<Shell>>();
    *shell.failure.lock().expect("never poisoned") = Some((title.to_string(), detail.to_string()));
    if let Some(window) = handle.get_webview_window("main") {
        replay_splash(&window);
    }
}

/// Tell the splash what it should say now. Run again when it finishes loading, so a message sent
/// before the page was ready is not lost.
pub(crate) fn replay_splash(window: &WebviewWindow) {
    let shell = window.state::<Arc<Shell>>();
    let script = if let Some((title, detail)) = shell.failure.lock().expect("never poisoned").clone() {
        format!(
            "window.rekallFail && window.rekallFail({}, {}, {})",
            js(&title),
            js(&detail),
            js(&log_file().to_string_lossy())
        )
    } else if let Some(text) = shell.status.lock().expect("never poisoned").clone() {
        format!("window.rekallStatus && window.rekallStatus({})", js(&text))
    } else {
        return;
    };
    if let Err(failed) = window.eval(&script) {
        error!(%failed, "Could not update the splash screen");
    }
}

fn js(text: &str) -> String {
    serde_json::to_string(text).unwrap_or_else(|_| "\"\"".into())
}

/// Once the splash has been on screen for `SPLASH_MINIMUM`: fades the window out, grows it unseen
/// to fill the screen, loads the console into it and fades it back in.
pub(crate) async fn dismiss_splash_then_open(handle: &AppHandle, shown_since: Instant) {
    tokio::time::sleep(SPLASH_MINIMUM.saturating_sub(shown_since.elapsed())).await;
    let Some(window) = handle.get_webview_window("main") else { return };
    if let Err(error) = bridges::fade_out(&window).await {
        error!(%error, "Could not fade the splash out");
    }
    if let Err(error) = grow_to_console(handle, &window).await {
        error!(%error, "Could not open the window maximized");
    }
    open_console(handle);
    // Fading in before the console has loaded shows the splash stretched to the full window.
    let shell = handle.state::<Arc<Shell>>();
    wait_until(|| shell.console_loaded.load(Ordering::SeqCst), CONSOLE_LOAD_LIMIT).await;
    if let Err(error) = bridges::fade_in(&window).await {
        error!(%error, "Could not fade the console in");
    }
}

/// Resizes the splash window to the console's restored size, then maximizes it. macOS applies a
/// resize on its own next pass through the run loop, and position and size do not necessarily land
/// together, so each step waits for the system to reach its target before the next one reads it.
async fn grow_to_console(handle: &AppHandle, window: &WebviewWindow) -> Result<(), String> {
    let failed = |error: tauri::Error| error.to_string();
    window.set_min_size(Some(CONSOLE_MINIMUM_SIZE)).map_err(failed)?;
    window.set_size(CONSOLE_SIZE).map_err(failed)?;
    let restored = CONSOLE_SIZE.to_physical::<u32>(window.scale_factor().map_err(failed)?);
    settle(|| window.inner_size().is_ok_and(|size| size == restored)).await;
    window.center().map_err(failed)?;
    let area = bridges::open_maximized(window, &handle.state::<bridges::WindowGeometry>())?;
    settle(|| window.outer_position().is_ok_and(|p| p == area.position) && window.inner_size().is_ok_and(|s| s == area.size)).await;
    Ok(())
}

/// Polls `reached` for up to half a second, in case the system never gets there.
async fn settle(reached: impl Fn() -> bool) {
    wait_until(reached, Duration::from_millis(500)).await;
}

/// Polls `reached` every 10 ms until it holds or `limit` passes.
async fn wait_until(reached: impl Fn() -> bool, limit: Duration) {
    let started = Instant::now();
    while !reached() && started.elapsed() < limit {
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

pub(crate) fn open_console(handle: &AppHandle) {
    let shell = handle.state::<Arc<Shell>>();
    shell.console_loaded.store(false, Ordering::SeqCst);
    *shell.status.lock().expect("never poisoned") = None;
    *shell.failure.lock().expect("never poisoned") = None;
    if let Some(window) = handle.get_webview_window("main") {
        if let Err(failed) = window.navigate(base()) {
            error!(%failed, "Could not open the console");
        }
    }
}
