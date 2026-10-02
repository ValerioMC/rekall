//! The question asked before the app quits while Claude sessions are still live: quitting stops
//! the server this process started, and with it every terminal it holds.

use std::time::Duration;

use serde::Deserialize;
use tauri::{AppHandle, Manager, Runtime, WebviewWindow};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tokio::sync::oneshot;
use tracing::warn;

const LOOKUP_TIMEOUT: Duration = Duration::from_secs(2);
const TITLE: &str = "Quit Rekall?";
const QUIT: &str = "Quit";
const STAY: &str = "Keep working";

/// The one field of a terminal that says whether its session is still running.
#[derive(Debug, Deserialize)]
pub struct SessionState {
    live: bool,
}

pub fn count_live(sessions: &[SessionState]) -> usize {
    sessions.iter().filter(|session| session.live).count()
}

pub fn warning(live: usize) -> String {
    if live == 1 {
        "1 Claude session is still running. Quitting stops the server and closes its terminal, so it will end and any work in progress is cut short.".into()
    } else {
        format!("{live} Claude sessions are still running. Quitting stops the server and closes their terminals, so they will end and any work in progress is cut short.")
    }
}

/// How many sessions the server on `port` reports as live.
pub async fn live_sessions(port: u16) -> Result<usize, reqwest::Error> {
    let client = reqwest::Client::builder().no_proxy().timeout(LOOKUP_TIMEOUT).build()?;
    let sessions: Vec<SessionState> = client
        .get(format!("http://127.0.0.1:{port}/api/terminals"))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(count_live(&sessions))
}

/// Whether the app may quit: at once when no session is live, otherwise as the dialog's answer.
/// A server that cannot be asked is not a reason to hold the user in the app, so it is logged and
/// the quit goes ahead.
pub async fn may_quit<R: Runtime>(handle: &AppHandle<R>, port: u16) -> bool {
    let live = match live_sessions(port).await {
        Ok(live) => live,
        Err(failure) => {
            warn!("Could not count the live sessions before quitting: {failure}");
            return true;
        }
    };
    if live == 0 {
        return true;
    }
    ask(handle, &warning(live)).await
}

async fn ask<R: Runtime>(handle: &AppHandle<R>, message: &str) -> bool {
    let (answer, reply) = oneshot::channel();
    let mut dialog = handle
        .dialog()
        .message(message)
        .title(TITLE)
        .kind(MessageDialogKind::Warning)
        .buttons(MessageDialogButtons::OkCancelCustom(QUIT.into(), STAY.into()));
    if let Some(window) = handle.get_webview_window("main") {
        dialog = dialog.parent(&window as &WebviewWindow<R>);
    }
    dialog.show(move |quit| {
        let _ = answer.send(quit);
    });
    reply.await.unwrap_or(false)
}

#[cfg(test)]
#[path = "../tests/unit/exit_guard_tests.rs"]
mod tests;
