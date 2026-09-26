use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use portable_pty::{ChildKiller, MasterPty, PtySize};
use rekall_common::{Id, Instant};
use rekall_service::claude::TerminalLaunch;
use tokio::sync::watch;
use tracing::debug;

use super::{Listener, TerminalView};
use super::scrollback::Scrollback;

pub(super) struct Terminal {
    pub(super) id: Id,
    pub(super) launch: TerminalLaunch,
    pub(super) step_id: Mutex<Option<Id>>,
    /// The last size the pane asked for, as (columns, rows). Its lock also serialises every
    /// resize, so the kick's delayed restore cannot undo a resize that landed meanwhile.
    pub(super) size: Mutex<(u16, u16)>,
    pub(super) skip_permissions: bool,
    pub(super) model: Option<String>,
    pub(super) effort: Option<String>,
    pub(super) started_at: Instant,
    pub(super) last_activity_at: Mutex<Instant>,
    pub(super) closing: AtomicBool,
    pub(super) master: Mutex<Box<dyn MasterPty + Send>>,
    pub(super) writer: Mutex<Box<dyn Write + Send>>,
    pub(super) killer: Mutex<Box<dyn ChildKiller + Send + Sync>>,
    pub(super) pid: Option<u32>,
    pub(super) exit: watch::Receiver<Option<i32>>,
    pub(super) listeners: Mutex<Vec<(u64, Arc<dyn Listener>)>>,
    pub(super) scrollback: Mutex<Scrollback>,
}

impl Terminal {
    pub(super) fn alive(&self) -> bool {
        self.exit.borrow().is_none()
    }

    pub(super) fn step_id(&self) -> Option<Id> {
        *self.step_id.lock().expect("never poisoned")
    }

    pub(super) fn view(&self) -> TerminalView {
        TerminalView {
            id: self.id,
            task_id: self.launch.task_id,
            step_id: self.step_id(),
            anchors: self.launch.anchors.clone(),
            working_dir: self.launch.working_dir.clone(),
            project_label: self.launch.project_label.clone(),
            task_label: self.launch.task_label.clone(),
            task_title: self.launch.task_title.clone(),
            skip_permissions: self.skip_permissions,
            model: self.model.clone(),
            effort: self.effort.clone(),
            live: self.alive() && !self.closing.load(Ordering::SeqCst),
            started_at: self.started_at,
            last_activity_at: *self.last_activity_at.lock().expect("never poisoned"),
        }
    }

    /// `Process.destroy()`: SIGTERM.
    pub(super) fn destroy(&self) {
        #[cfg(unix)]
        if let Some(pid) = self.pid {
            unsafe {
                libc::kill(pid as i32, libc::SIGTERM);
            }
            return;
        }
        let _ = self.killer.lock().expect("never poisoned").kill();
    }

    /// `Process.destroyForcibly()`: SIGKILL.
    pub(super) fn destroy_forcibly(&self) {
        #[cfg(unix)]
        if let Some(pid) = self.pid {
            unsafe {
                libc::kill(pid as i32, libc::SIGKILL);
            }
            return;
        }
        let _ = self.killer.lock().expect("never poisoned").kill();
    }

    pub(super) async fn wait_for(&self, timeout: Duration) -> bool {
        let mut exit = self.exit.clone();
        let exited = tokio::time::timeout(timeout, exit.wait_for(Option::is_some)).await.is_ok();
        exited
    }

    /// The exit code, or -1 while it is still running.
    pub(super) fn safe_exit_value(&self) -> i32 {
        self.exit.borrow().unwrap_or(-1)
    }

    pub(super) fn apply_win_size(&self, columns: u16, rows: u16) -> bool {
        let size = PtySize { rows, cols: columns, pixel_width: 0, pixel_height: 0 };
        match self.master.lock().expect("never poisoned").resize(size) {
            Ok(()) => true,
            Err(failed) => {
                debug!("Resize of terminal {} to {columns}x{rows} failed: {failed}", self.id);
                false
            }
        }
    }

    pub(super) fn notify_ended(&self, exit_code: i32, detail: &str) {
        let listeners = std::mem::take(&mut *self.listeners.lock().expect("never poisoned"));
        for (_, listener) in listeners {
            listener.ended(exit_code, detail);
        }
    }
}
