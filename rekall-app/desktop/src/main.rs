//! Rekall as a desktop application: one window over the same Axum server `rekall-server` runs,
//! started in this process. The window opens first on a splash screen, the server comes up
//! underneath it, and the console replaces the splash once it answers, as the macOS launcher did.
//!
//! The server is always a real HTTP server on the fixed port (`SERVER_PORT`, 47355): Claude Code
//! reaches `/mcp` there, and the console talks to it over the same REST, SSE and WebSocket routes
//! a browser uses. Only the three native bridges (`bridges.rs`) go through Tauri.
//!
//! If a Rekall server already answers on the port (a `rekall-server` in a terminal, another copy
//! of the app), the window attaches to it and leaves it running on quit: we did not start it, we
//! do not stop it.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bridges;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rekall_app::{AppConfig, Properties, Running, StartOptions};
use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder};
use tauri::webview::DownloadEvent;
use tauri::{AppHandle, Manager, RunEvent, Url, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use tauri_plugin_opener::OpenerExt;
use tracing::{error, info};

const DEFAULT_PORT: u16 = 47355;
const HOST: &str = "127.0.0.1";
const SLOW_NOTICE: Duration = Duration::from_secs(15);

/// What the shell keeps between events: the server it started (none when it attached to one), and
/// what the splash should say once it has loaded.
#[derive(Default)]
struct Shell {
    running: tokio::sync::Mutex<Option<Running>>,
    status: Mutex<Option<String>>,
    failure: Mutex<Option<(String, String)>>,
}

fn port() -> u16 {
    std::env::var("SERVER_PORT").ok().and_then(|p| p.trim().parse().ok()).unwrap_or(DEFAULT_PORT)
}

fn base() -> Url {
    Url::parse(&format!("http://{HOST}:{}/", port())).expect("a valid URL")
}

/// `~/Library/Logs/Rekall/server.log` on macOS, the data folder elsewhere.
fn log_file() -> PathBuf {
    #[cfg(target_os = "macos")]
    let folder = dirs::home_dir().unwrap_or_default().join("Library/Logs/Rekall");
    #[cfg(not(target_os = "macos"))]
    let folder = dirs::data_local_dir().unwrap_or_default().join("Rekall");
    folder.join("server.log")
}

/// Where `./data` resolves for the adoption of a legacy folder: a directory of the app's own
/// rather than whatever the process was launched from (`/` for an .app).
fn working_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    let folder = dirs::home_dir().unwrap_or_default().join("Library/Application Support/Rekall");
    #[cfg(not(target_os = "macos"))]
    let folder = dirs::data_dir().unwrap_or_default().join("Rekall");
    let _ = std::fs::create_dir_all(&folder);
    folder
}

fn init_logging() {
    let file = log_file();
    if let Some(parent) = file.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,sqlx=warn,sea_orm=warn,sea_orm_migration=warn"));
    match std::fs::File::create(&file) {
        Ok(log) => tracing_subscriber::fmt().with_env_filter(filter).with_ansi(false).with_writer(Mutex::new(log)).init(),
        Err(_) => tracing_subscriber::fmt().with_env_filter(filter).init(),
    }
}

fn main() {
    init_logging();
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .manage(Arc::new(Shell::default()))
        .manage(bridges::WindowGeometry::default())
        .invoke_handler(tauri::generate_handler![
            bridges::pick_folder,
            bridges::open_in_claude_code,
            bridges::notify,
            bridges::close_window,
            bridges::minimize_window,
            bridges::toggle_maximize_window
        ])
        .menu(|handle| {
            let reload = MenuItemBuilder::with_id("reload", "Reload").accelerator("CmdOrCtrl+R").build(handle)?;
            let open_log = MenuItemBuilder::with_id("open-log", "Open Server Log").build(handle)?;
            let app_menu = SubmenuBuilder::new(handle, "Rekall")
                .item(&PredefinedMenuItem::about(handle, Some("About Rekall"), None)?)
                .separator()
                .item(&PredefinedMenuItem::hide(handle, Some("Hide Rekall"))?)
                .separator()
                .item(&PredefinedMenuItem::quit(handle, Some("Quit Rekall"))?)
                .build()?;
            // Without an Edit menu the standard shortcuts never reach the web view, and the note
            // editor could not paste.
            let edit = SubmenuBuilder::new(handle, "Edit").undo().redo().separator().cut().copy().paste().select_all().build()?;
            let view = SubmenuBuilder::new(handle, "View")
                .item(&reload)
                .item(&open_log)
                .separator()
                .item(&PredefinedMenuItem::fullscreen(handle, Some("Enter Full Screen"))?)
                .build()?;
            MenuBuilder::new(handle).items(&[&app_menu, &edit, &view]).build()
        })
        .on_menu_event(|handle, event| match event.id().as_ref() {
            "reload" => reload(handle),
            "open-log" => {
                let _ = handle.opener().open_path(log_file().to_string_lossy(), None::<&str>);
            }
            _ => {}
        })
        .setup(|app| {
            build_window(app.handle())?;
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move { boot(handle).await });
            trap_signals(app.handle().clone());
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("the Rekall window");

    app.run(|handle, event| {
        if let RunEvent::Exit = event {
            // Graceful, so every terminal is closed and the database file is closed properly.
            let shell = handle.state::<Arc<Shell>>().inner().clone();
            tauri::async_runtime::block_on(async move {
                if let Some(running) = shell.running.lock().await.take() {
                    info!("Stopping the server");
                    let _ = tokio::time::timeout(rekall_app::server::STOP_WINDOW, running.stop()).await;
                }
            });
        }
    });
}

/// Quit and the window's close button reach `RunEvent::Exit`, but `kill` and a logout send a
/// signal nothing answers by default, which would take the process down with the server still
/// holding its database. Answer them with an orderly exit instead.
fn trap_signals(handle: AppHandle) {
    #[cfg(unix)]
    tauri::async_runtime::spawn(async move {
        use tokio::signal::unix::{signal, SignalKind};
        let (Ok(mut terminate), Ok(mut interrupt)) = (signal(SignalKind::terminate()), signal(SignalKind::interrupt())) else {
            return;
        };
        tokio::select! {
            _ = terminate.recv() => {}
            _ = interrupt.recv() => {}
        }
        handle.exit(0);
    });
    #[cfg(not(unix))]
    let _ = handle;
}

fn build_window(handle: &AppHandle) -> tauri::Result<WebviewWindow> {
    let opener = handle.clone();
    let downloads = handle.clone();
    let builder = WebviewWindowBuilder::new(handle, "main", WebviewUrl::App("index.html".into()))
        .title("Rekall")
        .inner_size(1440.0, 900.0)
        .min_inner_size(960.0, 600.0)
        .center()
        // Hidden until `open_maximized` below has already grown it to the screen, so the window
        // never flashes at this windowed size first.
        .visible(false)
        .theme(Some(tauri::Theme::Dark))
        .background_color(tauri::window::Color(8, 9, 12, 255))
        .shadow(false)
        .initialization_script(bridges::BRIDGE)
        // Anything not served by the local instance belongs in the browser, not in a window with
        // no address bar and no way back.
        .on_navigation(move |url| {
            if is_local(url) {
                return true;
            }
            let _ = opener.opener().open_url(url.as_str(), None::<&str>);
            false
        })
        // The export is a plain link to a zip: saved to Downloads under a name not yet taken, and
        // shown in the file manager once it is there.
        .on_download(move |_, event| {
            match event {
                DownloadEvent::Requested { destination, .. } => {
                    let name = destination
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_else(|| "download".into());
                    let folder = dirs::download_dir().unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join("Downloads"));
                    *destination = unused_path(&folder, &name);
                }
                DownloadEvent::Finished { path: Some(path), success: true, .. } => {
                    let _ = downloads.opener().reveal_item_in_dir(path);
                }
                _ => {}
            }
            true
        })
        .on_page_load(|window, payload| {
            if payload.event() == tauri::webview::PageLoadEvent::Finished && !is_server(payload.url()) {
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
    match bridges::open_maximized(&window, &handle.state::<bridges::WindowGeometry>()) {
        Ok(area) => {
            let window = window.clone();
            tauri::async_runtime::spawn(async move {
                // macOS applies the resize just issued on its own next pass through the run
                // loop rather than immediately, and position and size do not necessarily land
                // in that same pass together, so showing the window here would still risk
                // catching it mid-move. Wait for both to reach the target (a half-second cap in
                // case they somehow never do) before revealing it.
                for _ in 0..50 {
                    let there = window.outer_position().is_ok_and(|p| p == area.position)
                        && window.inner_size().is_ok_and(|s| s == area.size);
                    if there {
                        break;
                    }
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
                let _ = window.show();
            });
        }
        Err(error) => {
            error!("Could not open the window maximized: {error}");
            window.show()?;
        }
    }
    Ok(window)
}

fn is_local(url: &Url) -> bool {
    matches!(url.scheme(), "tauri" | "asset" | "about" | "data")
        || matches!(url.host_str(), Some("127.0.0.1" | "localhost" | "tauri.localhost"))
}

fn is_server(url: &Url) -> bool {
    matches!(url.host_str(), Some("127.0.0.1" | "localhost")) && url.port() == Some(port())
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

// ---------------------------------------------------------------- the splash screen

fn status(handle: &AppHandle, text: &str) {
    let shell = handle.state::<Arc<Shell>>();
    *shell.status.lock().unwrap() = Some(text.to_string());
    if let Some(window) = handle.get_webview_window("main") {
        replay_splash(&window);
    }
}

fn fail(handle: &AppHandle, title: &str, detail: &str) {
    error!("{title}: {detail}");
    let shell = handle.state::<Arc<Shell>>();
    *shell.failure.lock().unwrap() = Some((title.to_string(), detail.to_string()));
    if let Some(window) = handle.get_webview_window("main") {
        replay_splash(&window);
    }
}

/// Tell the splash what it should say now. Run again when it finishes loading, so a message sent
/// before the page was ready is not lost.
fn replay_splash(window: &WebviewWindow) {
    let shell = window.state::<Arc<Shell>>();
    let script = if let Some((title, detail)) = shell.failure.lock().unwrap().clone() {
        format!(
            "window.rekallFail && window.rekallFail({}, {}, {})",
            js(&title),
            js(&detail),
            js(&log_file().to_string_lossy())
        )
    } else if let Some(text) = shell.status.lock().unwrap().clone() {
        format!("window.rekallStatus && window.rekallStatus({})", js(&text))
    } else {
        return;
    };
    let _ = window.eval(&script);
}

fn js(text: &str) -> String {
    serde_json::to_string(text).unwrap_or_else(|_| "\"\"".into())
}

// ---------------------------------------------------------------- the server

/// Attach to a server that answers, or start one; then show the console.
async fn boot(handle: AppHandle) {
    if answering().await {
        status(&handle, "Connecting to the running instance…");
        open_console(&handle);
        return;
    }
    status(&handle, "Starting the server…");
    let args = vec![format!("--server.port={}", port())];
    let mut config = AppConfig::from_sources(&Properties::new(&args, std::env::vars().collect()));
    config.working_dir = working_dir();

    let starting = rekall_app::start(config, StartOptions::default());
    tokio::pin!(starting);
    let started = tokio::select! {
        started = &mut starting => started,
        _ = tokio::time::sleep(SLOW_NOTICE) => {
            status(&handle, "Still starting. The first run creates the database…");
            starting.await
        }
    };
    match started {
        Ok(running) => {
            *handle.state::<Arc<Shell>>().running.lock().await = Some(running);
            open_console(&handle);
        }
        Err(failed) => fail(
            &handle,
            "The server did not start",
            &format!("{failed}. The most common cause is port {} already being used by something that is not Rekall.", port()),
        ),
    }
}

fn open_console(handle: &AppHandle) {
    let shell = handle.state::<Arc<Shell>>();
    *shell.status.lock().unwrap() = None;
    *shell.failure.lock().unwrap() = None;
    if let Some(window) = handle.get_webview_window("main") {
        let _ = window.navigate(base());
    }
}

/// True when something that looks like Rekall answers: 200 is a healthy server, 503 one whose
/// database is unreachable (the console has a screen for that). Anything else is not Rekall.
async fn answering() -> bool {
    let Ok(client) = reqwest::Client::builder().no_proxy().timeout(Duration::from_secs(2)).build() else { return false };
    match client.get(format!("http://{HOST}:{}/actuator/health", port())).send().await {
        Ok(response) => matches!(response.status().as_u16(), 200 | 503),
        Err(_) => false,
    }
}

fn reload(handle: &AppHandle) {
    let Some(window) = handle.get_webview_window("main") else { return };
    if window.url().is_ok_and(|url| is_server(&url)) {
        let _ = window.eval("window.location.reload()");
        return;
    }
    let shell = handle.state::<Arc<Shell>>();
    *shell.failure.lock().unwrap() = None;
    let _ = window.navigate(Url::parse("tauri://localhost/index.html").unwrap_or_else(|_| base()));
    let handle = handle.clone();
    tauri::async_runtime::spawn(async move {
        let already = handle.state::<Arc<Shell>>().running.lock().await.is_some();
        if already {
            open_console(&handle);
        } else {
            boot(handle).await;
        }
    });
}
