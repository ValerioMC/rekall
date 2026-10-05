//! Rekall as a desktop application: one window over the same Axum server `rekall-server` runs,
//! started in this process. The window opens first on a splash screen, the server comes up
//! underneath it, and the console replaces the splash once it answers, as the macOS launcher did.
//!
//! The server is always a real HTTP server on the fixed port (`SERVER_PORT`, 47355): Claude Code
//! reaches `/mcp` there, and the console talks to it over the same REST, SSE and WebSocket routes
//! a browser uses. Only the native bridges (`bridges`, `update_install`) go through Tauri.
//!
//! If a Rekall server already answers on the port (a `rekall-server` in a terminal, another copy
//! of the app), the window attaches to it and leaves it running on quit: we did not start it, we
//! do not stop it.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bridges;
mod download_tracker;
mod exit_guard;
mod main_window;
mod server_boot;
mod shell;
mod splash;
mod update_install;

use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder};
use tauri::{AppHandle, Manager, RunEvent};
use tauri_plugin_opener::OpenerExt;
use tracing::info;

use main_window::build_window;
use server_boot::{boot, reload};
use shell::{log_file, port, Shell};

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
        .manage(exit_guard::LeaveQuestion::default())
        .invoke_handler(tauri::generate_handler![
            bridges::folder_picker::pick_folder,
            bridges::claude_code_launcher::open_in_claude_code,
            bridges::notifier::notify,
            bridges::window_controls::close_window,
            bridges::window_controls::minimize_window,
            bridges::window_controls::toggle_maximize_window,
            update_install::install_update,
            exit_guard::answer_leave
        ])
        .menu(|handle| {
            let reload = MenuItemBuilder::with_id("reload", "Reload").accelerator("CmdOrCtrl+R").build(handle)?;
            // Not the predefined item: on macOS that one ends the process without asking, and the
            // app has to ask first when a Claude session is still live.
            let quit = MenuItemBuilder::with_id("quit", "Quit Rekall").accelerator("CmdOrCtrl+Q").build(handle)?;
            let open_log = MenuItemBuilder::with_id("open-log", "Open Server Log").build(handle)?;
            let app_menu = SubmenuBuilder::new(handle, "Rekall")
                .item(&PredefinedMenuItem::about(handle, Some("About Rekall"), None)?)
                .separator()
                .item(&PredefinedMenuItem::hide(handle, Some("Hide Rekall"))?)
                .separator()
                .item(&quit)
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
            "quit" => request_quit(handle),
            "open-log" => {
                let _ = handle.opener().open_path(log_file().to_string_lossy(), None::<&str>);
            }
            _ => {}
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                request_quit(window.app_handle());
            }
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

/// Quit and the window's close button both come here. When this process started the server, a live
/// Claude session dies with it, so the user is asked first; an attached server keeps its sessions
/// after we leave, so nothing is asked.
fn request_quit(handle: &AppHandle) {
    let shell = handle.state::<Arc<Shell>>().inner().clone();
    if shell.quitting.swap(true, Ordering::SeqCst) {
        return;
    }
    let handle = handle.clone();
    tauri::async_runtime::spawn(async move {
        let owns_server = shell.running.lock().await.is_some();
        if !owns_server || exit_guard::may_leave(&handle, port(), exit_guard::Leaving::Quit).await {
            handle.exit(0);
        } else {
            shell.quitting.store(false, Ordering::SeqCst);
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
