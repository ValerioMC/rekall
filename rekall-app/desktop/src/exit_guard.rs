//! The question asked before the app quits, or restarts into an update, while Claude sessions are
//! still live: either stops the server this process started, and with it every terminal it holds.
//! The console asks it in its own dialog; the system dialog is the fallback when the console
//! cannot show one.

use std::sync::Mutex;
use std::time::Duration;

use serde::Deserialize;
use tauri::{AppHandle, Manager, Runtime, State, WebviewWindow};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tokio::sync::oneshot;
use tracing::warn;

const LOOKUP_TIMEOUT: Duration = Duration::from_secs(2);
const STAY: &str = "Keep working";

/// Why the server is about to stop: it decides the dialog's title, button and wording.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Leaving {
    Quit,
    Update,
}

impl Leaving {
    fn title(self) -> &'static str {
        match self {
            Leaving::Quit => "Quit Rekall?",
            Leaving::Update => "Install the update now?",
        }
    }

    fn confirm(self) -> &'static str {
        match self {
            Leaving::Quit => "Quit",
            Leaving::Update => "Install and restart",
        }
    }
}

/// The question the console is answering now. `None` as the answer means the console could not
/// show it, so the system dialog asks instead.
#[derive(Default)]
pub struct LeaveQuestion {
    pending: Mutex<Option<oneshot::Sender<Option<bool>>>>,
}

impl LeaveQuestion {
    /// Starts a question; one still waiting is abandoned and counts as a refusal.
    fn open(&self) -> oneshot::Receiver<Option<bool>> {
        let (sender, receiver) = oneshot::channel();
        *self.pending.lock().expect("the leave question lock") = Some(sender);
        receiver
    }

    /// Delivers the answer to the question that is waiting; false when none is.
    fn answer(&self, answer: Option<bool>) -> bool {
        let sender = self.pending.lock().expect("the leave question lock").take();
        sender.is_some_and(|sender| sender.send(answer).is_ok())
    }
}

/// The `answerLeave(confirmed)` bridge: the console's reply to the question it was shown.
#[tauri::command]
pub fn answer_leave(question: State<'_, LeaveQuestion>, answer: Option<bool>) {
    question.answer(answer);
}

/// The one field of a terminal that says whether its session is still running.
#[derive(Debug, Deserialize)]
pub struct SessionState {
    live: bool,
}

pub fn count_live(sessions: &[SessionState]) -> usize {
    sessions.iter().filter(|session| session.live).count()
}

pub fn warning(live: usize, leaving: Leaving) -> String {
    let action = match leaving {
        Leaving::Quit => "Quitting",
        Leaving::Update => "Installing the update restarts Rekall, which",
    };
    if live == 1 {
        format!("1 Claude session is still running. {action} stops the server and closes its terminal, so it will end and any work in progress is cut short.")
    } else {
        format!("{live} Claude sessions are still running. {action} stops the server and closes their terminals, so they will end and any work in progress is cut short.")
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

/// Whether the server may stop: at once when no session is live, otherwise as the dialog's answer.
/// A server that cannot be asked is not a reason to hold the user in the app, so it is logged and
/// the stop goes ahead.
pub async fn may_leave<R: Runtime>(handle: &AppHandle<R>, port: u16, leaving: Leaving) -> bool {
    let live = match live_sessions(port).await {
        Ok(live) => live,
        Err(failure) => {
            warn!("Could not count the live sessions before stopping the server: {failure}");
            return true;
        }
    };
    if live == 0 {
        return true;
    }
    ask(handle, leaving, &warning(live, leaving)).await
}

async fn ask<R: Runtime>(handle: &AppHandle<R>, leaving: Leaving, message: &str) -> bool {
    match ask_in_console(handle, leaving, message).await {
        Some(confirmed) => confirmed,
        None => ask_with_system_dialog(handle, leaving, message).await,
    }
}

/// `None` when the console cannot show the question.
async fn ask_in_console<R: Runtime>(handle: &AppHandle<R>, leaving: Leaving, message: &str) -> Option<bool> {
    let window = handle.get_webview_window("main")?;
    let reply = handle.state::<LeaveQuestion>().open();
    if window.eval(leave_script(leaving, message)).is_err() {
        handle.state::<LeaveQuestion>().answer(None);
    }
    reply.await.unwrap_or(Some(false))
}

/// Hands the question to the console; a console not yet listening answers `null` at once.
fn leave_script(leaving: Leaving, message: &str) -> String {
    let detail = serde_json::json!({
        "title": leaving.title(),
        "message": message,
        "confirm": leaving.confirm(),
        "stay": STAY,
    });
    format!(
        "window.rekallLeaveReady ? window.dispatchEvent(new CustomEvent('rekall:leave', {{ detail: {detail} }})) : window.rekallDesktop.answerLeave(null)"
    )
}

async fn ask_with_system_dialog<R: Runtime>(handle: &AppHandle<R>, leaving: Leaving, message: &str) -> bool {
    let (answer, reply) = oneshot::channel();
    let mut dialog = handle
        .dialog()
        .message(message)
        .title(leaving.title())
        .kind(MessageDialogKind::Warning)
        .buttons(MessageDialogButtons::OkCancelCustom(leaving.confirm().into(), STAY.into()));
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
