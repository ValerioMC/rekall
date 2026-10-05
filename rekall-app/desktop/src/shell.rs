//! What every part of the shell shares: its state between events, the port it talks to, and
//! where its files live.

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use rekall_app::Running;
use tauri::Url;
use tracing::warn;

const DEFAULT_PORT: u16 = 47355;
pub(crate) const HOST: &str = "127.0.0.1";

/// What the shell keeps between events: the server it started (none when it attached to one), and
/// what the splash should say once it has loaded. `quitting` is set while the quit question is on
/// screen, and stays set once the answer is yes.
#[derive(Default)]
pub(crate) struct Shell {
    pub(crate) running: tokio::sync::Mutex<Option<Running>>,
    pub(crate) quitting: AtomicBool,
    pub(crate) console_loaded: AtomicBool,
    pub(crate) status: Mutex<Option<String>>,
    pub(crate) failure: Mutex<Option<(String, String)>>,
}

pub(crate) fn port() -> u16 {
    std::env::var("SERVER_PORT").ok().and_then(|p| p.trim().parse().ok()).unwrap_or(DEFAULT_PORT)
}

pub(crate) fn base() -> Url {
    Url::parse(&format!("http://{HOST}:{}/", port())).expect("a valid URL")
}

/// `~/Library/Logs/Rekall/server.log` on macOS, the data folder elsewhere.
pub(crate) fn log_file() -> PathBuf {
    #[cfg(target_os = "macos")]
    let folder = dirs::home_dir().unwrap_or_default().join("Library/Logs/Rekall");
    #[cfg(not(target_os = "macos"))]
    let folder = dirs::data_local_dir().unwrap_or_default().join("Rekall");
    folder.join("server.log")
}

/// Where `./data` resolves when an existing database there is adopted: a directory of the app's own
/// rather than whatever the process was launched from (`/` for an .app).
pub(crate) fn working_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    let folder = dirs::home_dir().unwrap_or_default().join("Library/Application Support/Rekall");
    #[cfg(not(target_os = "macos"))]
    let folder = dirs::data_dir().unwrap_or_default().join("Rekall");
    if let Err(failed) = std::fs::create_dir_all(&folder) {
        warn!(%failed, folder = %folder.display(), "Could not create the app's working folder");
    }
    folder
}

pub(crate) fn is_server(url: &Url) -> bool {
    matches!(url.host_str(), Some("127.0.0.1" | "localhost")) && url.port() == Some(port())
}
