//! The one window: hidden until it fades in at its splash size, with navigation outside the local
//! instance sent to the browser and downloads saved to the Downloads folder.

use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use tauri::webview::DownloadEvent;
use tauri::{AppHandle, LogicalSize, Manager, Url, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use tauri_plugin_opener::OpenerExt;
use tracing::error;

use crate::bridges;
use crate::download_tracker::DownloadTracker;
use crate::shell::{is_server, Shell};
use crate::splash::replay_splash;

pub(crate) fn build_window(handle: &AppHandle) -> tauri::Result<WebviewWindow> {
    let opener = handle.clone();
    let downloads = handle.clone();
    let tracker = DownloadTracker::default();
    let splash = splash_window_size(handle);
    let builder = WebviewWindowBuilder::new(handle, "main", WebviewUrl::App("index.html".into()))
        .title("Rekall")
        .inner_size(splash.width, splash.height)
        .center()
        // Hidden until it can fade in at its splash size.
        .visible(false)
        .theme(Some(tauri::Theme::Dark))
        .background_color(tauri::window::Color(8, 9, 12, 255))
        .shadow(false)
        .initialization_script(bridges::bridge_script())
        // Anything not served by the local instance belongs in the browser, not in a window with
        // no address bar and no way back.
        .on_navigation(move |url| {
            if is_local(url) {
                return true;
            }
            let _ = opener.opener().open_url(url.as_str(), None::<&str>);
            false
        })
        // A download is a plain link (a backup's zip): saved to Downloads under a name not yet taken,
        // and shown in the file manager once it is there.
        .on_download(move |_, event| {
            match event {
                DownloadEvent::Requested { url, destination } => {
                    let name = destination
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_else(|| "download".into());
                    let folder = dirs::download_dir().unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join("Downloads"));
                    *destination = unused_path(&folder, &name);
                    tracker.started(url.as_str(), destination);
                }
                DownloadEvent::Finished { url, path, success } => {
                    let saved = tracker.finished(url.as_str(), path);
                    match (success, saved) {
                        (true, Some(path)) => {
                            if let Err(error) = downloads.opener().reveal_item_in_dir(&path) {
                                error!("Could not show {} in the file manager: {error}", path.display());
                            }
                        }
                        (true, None) => error!("Download of {url} finished with no known destination"),
                        (false, _) => error!("Download of {url} failed"),
                    }
                }
                _ => {}
            }
            true
        })
        .on_page_load(|window, payload| {
            if payload.event() != tauri::webview::PageLoadEvent::Finished {
                return;
            }
            if is_server(payload.url()) {
                window.state::<Arc<Shell>>().console_loaded.store(true, Ordering::SeqCst);
            } else {
                replay_splash(&window);
            }
        });
    // No native title bar and no native traffic lights either: the overlay title bar still drew
    // the system's own close/minimize/maximize buttons, which sat oddly over the console's header.
    // Fully borderless instead, so the only close/minimize/maximize buttons are the ones the
    // console draws itself in AnchorBar.vue, wired to `close_window`/`minimize_window`/
    // `toggle_maximize_window` in bridges.rs. The window stays resizable and miniaturizable: only
    // the titlebar chrome is gone. The header itself is the drag region
    // (`data-tauri-drag-region` in AnchorBar.vue), and double-clicking it maximizes, both for free
    // from Tauri's own drag-region handling.
    #[cfg(target_os = "macos")]
    let builder = builder.decorations(false);
    let window = builder.build()?;
    #[cfg(target_os = "macos")]
    if let Err(error) = bridges::allow_native_fullscreen(&window) {
        error!("Could not enable native full screen: {error}");
    }
    let visible = window.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(error) = bridges::show_fading_in(&visible).await {
            error!("Could not fade the window in: {error}");
            let _ = visible.show();
        }
    });
    Ok(window)
}

/// The splash card for the monitor the cursor-less launch lands on (the primary one), or a
/// sensible default when no monitor answers.
fn splash_window_size(handle: &AppHandle) -> LogicalSize<f64> {
    match handle.primary_monitor() {
        Ok(Some(monitor)) => bridges::splash_size(monitor.work_area().size.width, monitor.scale_factor()),
        _ => LogicalSize::new(768.0, 429.0),
    }
}

fn is_local(url: &Url) -> bool {
    matches!(url.scheme(), "tauri" | "asset" | "about" | "data")
        || matches!(url.host_str(), Some("127.0.0.1" | "localhost" | "tauri.localhost"))
}

fn unused_path(folder: &std::path::Path, name: &str) -> PathBuf {
    let candidate = folder.join(name);
    if !candidate.exists() {
        return candidate;
    }
    let path = std::path::Path::new(name);
    let stem = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    let extension = path.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
    (1..=999)
        .map(|index| folder.join(format!("{stem}-{index}{extension}")))
        .find(|next| !next.exists())
        .unwrap_or_else(|| folder.join(format!("{stem}-{}{extension}", uuid::Uuid::new_v4())))
}
