//! A terminal's process: the PTY it runs in, the thread that pumps its output to the scrollback
//! and the panes, the thread that waits for it to exit, and the resize that makes it repaint.

use std::io::Read;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use rekall_common::{Id, Instant};
use rekall_service::claude::TerminalLaunch;
use tokio::sync::watch;
use tracing::{debug, warn};

use super::claude_command::ClaudeOptions;
use super::scrollback::Scrollback;
use super::terminal::Terminal;
use super::Listener;

const DEFAULT_COLUMNS: u16 = 80;
const DEFAULT_ROWS: u16 = 24;
const READ_BUFFER: usize = 8192;
/// How long the repaint kick holds the PTY a row short: long enough for claude's event loop to
/// read the smaller size, or it coalesces both SIGWINCHs into no change and never repaints.
const KICK_SETTLE: Duration = Duration::from_millis(150);

/// Starts `command` in a fresh PTY. `on_exit` runs on the waiting thread once the process has
/// ended and its exit code is recorded.
pub(super) fn spawn_terminal(
    id: Id,
    launch: TerminalLaunch,
    step_id: Option<Id>,
    options: ClaudeOptions,
    command: CommandBuilder,
    scrollback_bytes: usize,
    on_exit: impl FnOnce() + Send + 'static,
) -> Result<Arc<Terminal>, String> {
    let text = |e: &dyn std::fmt::Display| e.to_string();
    let pair = native_pty_system()
        .openpty(PtySize { rows: DEFAULT_ROWS, cols: DEFAULT_COLUMNS, pixel_width: 0, pixel_height: 0 })
        .map_err(|e| text(&e))?;
    let mut child = pair.slave.spawn_command(command).map_err(|e| text(&e))?;
    drop(pair.slave);
    let reader = pair.master.try_clone_reader().map_err(|e| text(&e))?;
    let writer = pair.master.take_writer().map_err(|e| text(&e))?;
    let killer = child.clone_killer();
    let pid = child.process_id();
    let (exit_sender, exit) = watch::channel(None);
    let now = Instant::now();
    let terminal = Arc::new(Terminal {
        id,
        launch,
        step_id: Mutex::new(step_id),
        size: Mutex::new((DEFAULT_COLUMNS, DEFAULT_ROWS)),
        skip_permissions: options.skip_permissions,
        model: options.model,
        effort: options.effort,
        started_at: now,
        last_activity_at: Mutex::new(now),
        closing: AtomicBool::new(false),
        master: Mutex::new(pair.master),
        writer: Mutex::new(writer),
        killer: Mutex::new(killer),
        pid,
        exit,
        listeners: Mutex::new(Vec::new()),
        scrollback: Mutex::new(Scrollback::new(scrollback_bytes)),
    });

    let pumped = terminal.clone();
    std::thread::Builder::new()
        .name(format!("terminal-pty-{id}"))
        .spawn(move || pump(&pumped, reader))
        .map_err(|e| text(&e))?;

    std::thread::Builder::new()
        .name(format!("terminal-wait-{id}"))
        .spawn(move || {
            let code = match child.wait() {
                Ok(status) => status.exit_code() as i32,
                Err(_) => -1,
            };
            let _ = exit_sender.send(Some(code));
            on_exit();
        })
        .map_err(|e| text(&e))?;
    Ok(terminal)
}

fn pump(terminal: &Terminal, mut reader: Box<dyn Read + Send>) {
    let mut buffer = vec![0u8; READ_BUFFER];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(read) => {
                let chunk = &buffer[..read];
                terminal.scrollback.lock().expect("never poisoned").append(chunk);
                let listeners: Vec<Arc<dyn Listener>> =
                    terminal.listeners.lock().expect("never poisoned").iter().map(|(_, l)| l.clone()).collect();
                for listener in listeners {
                    listener.output(chunk);
                }
            }
            Err(ended) => {
                debug!(%ended, terminal_id = %terminal.id, "PTY output closed");
                break;
            }
        }
    }
}

/// Wobble the PTY down a row and, after `KICK_SETTLE`, back to its last known size: each step is
/// a size change claude observes, so it repaints in full even when the pane's own resize is a no-op.
pub(super) fn kick_resize(terminal: Arc<Terminal>) {
    {
        let (columns, rows) = *terminal.size.lock().expect("never poisoned");
        if rows <= 1 || !terminal.apply_win_size(columns, rows - 1) {
            return;
        }
    }
    let restoring = terminal.clone();
    let delayed = std::thread::Builder::new().name(format!("terminal-kick-{}", terminal.id)).spawn(move || {
        std::thread::sleep(KICK_SETTLE);
        restore_size(&restoring);
    });
    if let Err(failed) = delayed {
        warn!(%failed, terminal_id = %terminal.id, "Could not delay the repaint kick");
        restore_size(&terminal);
    }
}

fn restore_size(terminal: &Terminal) {
    let size = terminal.size.lock().expect("never poisoned");
    terminal.apply_win_size(size.0, size.1);
}
