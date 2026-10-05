//! Attaching to a Rekall server that already answers on the port, or starting one in this
//! process, and reloading the window onto whichever it is.

use std::sync::Arc;
use std::time::{Duration, Instant};

use rekall_app::{AppConfig, Properties, StartOptions};
use tauri::{AppHandle, Manager, Url};

use crate::shell::{base, is_server, port, working_dir, Shell, HOST};
use crate::splash::{dismiss_splash_then_open, fail, open_console, status};

const SLOW_NOTICE: Duration = Duration::from_secs(15);

/// The configuration the in-process server starts with, read again by whatever needs its settings.
pub(crate) fn server_config() -> AppConfig {
    let args = vec![format!("--server.port={}", port())];
    let mut config = AppConfig::from_sources(&Properties::new(&args, std::env::vars().collect()));
    config.working_dir = working_dir();
    config
}

/// Attach to a server that answers, or start one; then show the console.
pub(crate) async fn boot(handle: AppHandle) {
    let since = Instant::now();
    if answering().await {
        status(&handle, "Connecting to the running instance…");
        dismiss_splash_then_open(&handle, since).await;
        return;
    }
    status(&handle, "Starting the server…");
    let starting = rekall_app::start(server_config(), StartOptions::default());
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
            dismiss_splash_then_open(&handle, since).await;
        }
        Err(failed) => fail(
            &handle,
            "The server did not start",
            &format!("{failed}. The most common cause is port {} already being used by something that is not Rekall.", port()),
        ),
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

pub(crate) fn reload(handle: &AppHandle) {
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
