//! Owns the `claude` pseudo-terminals behind the in-app terminal pane: one PTY per task, each
//! running the interactive `claude` TUI. A reader thread pumps PTY output to every attached
//! `Listener` and into a bounded scrollback; `write` and `resize` carry input the other way. The
//! terminal moves the task's checklist marker (a step to `RUNNING` and back) or its review line as
//! it opens and closes, and announces every end, however it came, as a `TerminalEnded` event.
//! Nothing is persisted; strays are bounded by a cap, an idle sweep, and a shutdown hook.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU16, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::TimeDelta;
use portable_pty::{native_pty_system, ChildKiller, CommandBuilder, MasterPty, PtySize};
use rekall_common::{Id, Instant, RekallError, Result};
use rekall_service::claude::{TerminalLaunch, TerminalLaunchService};
use rekall_service::review::TaskReviewService;
use rekall_service::step::TaskStepService;
use serde::Serialize;
use tokio::sync::{broadcast, watch};
use tracing::{debug, info, warn};

use crate::cli::ClaudeCli;
use crate::config::ClaudeConfig;
use crate::terminal::TerminalMode;

const DEFAULT_COLUMNS: u16 = 80;
const DEFAULT_ROWS: u16 = 24;
const MAX_DIMENSION: i64 = 1000;
const READ_BUFFER: usize = 8192;

/// Only the aliases the settings offer are accepted; anything else leaves the account default.
pub const MODEL_ALIASES: [&str; 4] = ["opus", "sonnet", "haiku", "fable"];
pub const EFFORT_LEVELS: [&str; 5] = ["low", "medium", "high", "xhigh", "max"];

/// A sink for one terminal's output and the moment it ends. Implemented by the socket handler.
pub trait Listener: Send + Sync {
    fn output(&self, data: &[u8]);
    fn ended(&self, exit_code: i32, detail: &str);
}

/// A terminal has gone, whether it exited, was closed, idled out or went down with the app.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TerminalEnded {
    pub terminal_id: Id,
    pub task_id: Id,
}

/// A live terminal as the console sees it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalView {
    pub id: Id,
    pub task_id: Id,
    pub step_id: Option<Id>,
    pub anchors: String,
    pub working_dir: String,
    pub project_label: String,
    pub task_label: String,
    pub task_title: String,
    pub skip_permissions: bool,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub live: bool,
    pub started_at: Instant,
    pub last_activity_at: Instant,
}

/// A fixed-size ring of recent PTY bytes so a late-opening pane repaints. A snapshot may start
/// mid escape-sequence and flicker once on attach.
struct Scrollback {
    ring: Vec<u8>,
    size: usize,
    start: usize,
}

impl Scrollback {
    fn new(capacity: usize) -> Self {
        Self { ring: vec![0; capacity.max(1)], size: 0, start: 0 }
    }

    fn append(&mut self, data: &[u8]) {
        let capacity = self.ring.len();
        for byte in data {
            self.ring[(self.start + self.size) % capacity] = *byte;
            if self.size < capacity {
                self.size += 1;
            } else {
                self.start = (self.start + 1) % capacity;
            }
        }
    }

    fn snapshot(&self) -> Vec<u8> {
        let capacity = self.ring.len();
        (0..self.size).map(|at| self.ring[(self.start + at) % capacity]).collect()
    }
}

struct Terminal {
    id: Id,
    launch: TerminalLaunch,
    step_id: Mutex<Option<Id>>,
    columns: AtomicU16,
    rows: AtomicU16,
    skip_permissions: bool,
    model: Option<String>,
    effort: Option<String>,
    started_at: Instant,
    last_activity_at: Mutex<Instant>,
    closing: AtomicBool,
    master: Mutex<Box<dyn MasterPty + Send>>,
    writer: Mutex<Box<dyn Write + Send>>,
    killer: Mutex<Box<dyn ChildKiller + Send + Sync>>,
    pid: Option<u32>,
    exit: watch::Receiver<Option<i32>>,
    listeners: Mutex<Vec<(u64, Arc<dyn Listener>)>>,
    scrollback: Mutex<Scrollback>,
}

impl Terminal {
    fn alive(&self) -> bool {
        self.exit.borrow().is_none()
    }

    fn step_id(&self) -> Option<Id> {
        *self.step_id.lock().expect("never poisoned")
    }

    fn view(&self) -> TerminalView {
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
    fn destroy(&self) {
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
    fn destroy_forcibly(&self) {
        #[cfg(unix)]
        if let Some(pid) = self.pid {
            unsafe {
                libc::kill(pid as i32, libc::SIGKILL);
            }
            return;
        }
        let _ = self.killer.lock().expect("never poisoned").kill();
    }

    async fn wait_for(&self, timeout: Duration) -> bool {
        let mut exit = self.exit.clone();
        let exited = tokio::time::timeout(timeout, exit.wait_for(Option::is_some)).await.is_ok();
        exited
    }

    /// The exit code, or -1 while it is still running.
    fn safe_exit_value(&self) -> i32 {
        self.exit.borrow().unwrap_or(-1)
    }

    fn apply_win_size(&self, columns: u16, rows: u16) -> bool {
        let size = PtySize { rows, cols: columns, pixel_width: 0, pixel_height: 0 };
        match self.master.lock().expect("never poisoned").resize(size) {
            Ok(()) => true,
            Err(failed) => {
                debug!("Resize of terminal {} to {columns}x{rows} failed: {failed}", self.id);
                false
            }
        }
    }

    fn notify_ended(&self, exit_code: i32, detail: &str) {
        let listeners = std::mem::take(&mut *self.listeners.lock().expect("never poisoned"));
        for (_, listener) in listeners {
            listener.ended(exit_code, detail);
        }
    }
}

pub struct PtyTerminalManager {
    launch_service: TerminalLaunchService,
    cli: ClaudeCli,
    task_steps: TaskStepService,
    task_review: TaskReviewService,
    ended: broadcast::Sender<TerminalEnded>,
    max_live: usize,
    idle_minutes: u64,
    scrollback_bytes: usize,
    live: Mutex<HashMap<Id, Arc<Terminal>>>,
    /// Serialises the cap check and registration so two fast opens cannot both slip past the cap.
    spawn_gate: tokio::sync::Mutex<()>,
    next_listener: AtomicU64,
    stopped: AtomicBool,
}

impl PtyTerminalManager {
    pub fn new(
        launch_service: TerminalLaunchService,
        cli: ClaudeCli,
        task_steps: TaskStepService,
        task_review: TaskReviewService,
        config: &ClaudeConfig,
    ) -> Arc<Self> {
        let (ended, _) = broadcast::channel(256);
        Arc::new(Self {
            launch_service,
            cli,
            task_steps,
            task_review,
            ended,
            max_live: config.max_sessions,
            idle_minutes: config.idle_minutes,
            scrollback_bytes: config.scrollback_bytes,
            live: Mutex::new(HashMap::new()),
            spawn_gate: tokio::sync::Mutex::new(()),
            next_listener: AtomicU64::new(0),
            stopped: AtomicBool::new(false),
        })
    }

    /// Every end of every terminal, for the run queue.
    pub fn subscribe_ended(&self) -> broadcast::Receiver<TerminalEnded> {
        self.ended.subscribe()
    }

    /// Sweep idle terminals every `sweep_minutes`.
    pub fn start_reaper(self: &Arc<Self>, sweep_minutes: u64) {
        let this = Arc::downgrade(self);
        let every = Duration::from_secs(sweep_minutes.max(1) * 60);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval_at(tokio::time::Instant::now() + every, every);
            loop {
                interval.tick().await;
                let Some(manager) = this.upgrade() else { return };
                if manager.stopped.load(Ordering::SeqCst) {
                    return;
                }
                manager.sweep_idle().await;
            }
        });
    }

    fn live_map(&self) -> std::sync::MutexGuard<'_, HashMap<Id, Arc<Terminal>>> {
        self.live.lock().expect("never poisoned")
    }

    // ---------------------------------------------------------------- commands

    /// Start a `claude` PTY for this task in its project's folder, with the `/rk` line of `mode`
    /// typed first. A missing folder or CLI is a retriable conflict. A step id is marked
    /// `RUNNING` while the terminal is on it; none means the task itself. A plan is always on the
    /// task, so [`TerminalMode::Plan`] drops the step id. A second open on a task that already has
    /// a terminal reuses it: a work open is retargeted, a plan open has its plan line typed into
    /// that session.
    #[allow(clippy::too_many_arguments)]
    pub async fn open(
        self: &Arc<Self>,
        task_id: Id,
        step_id: Option<Id>,
        skip_permissions: bool,
        model: Option<&str>,
        effort: Option<&str>,
        mode: TerminalMode,
    ) -> Result<TerminalView> {
        let launch = self.launch_service.resolve(task_id).await?;
        let step_id = if mode == TerminalMode::Plan { None } else { step_id };

        if let Some(existing) = self.live_terminal_for_task(task_id) {
            if mode == TerminalMode::Plan {
                self.write(existing.id, format!("{}\r", mode.first_line(&launch.anchors)).as_bytes())?;
                return Ok(existing.view());
            }
            return Ok(self.retarget(&existing, step_id).await);
        }

        let directory = Path::new(&launch.working_dir);
        if !directory.is_dir() {
            return Err(RekallError::conflict(format!(
                "This project's folder ({}) is not there. Set it again on the project page.",
                launch.working_dir
            )));
        }
        let Some(binary) = self.cli.locate().await else {
            return Err(RekallError::conflict(
                "Claude Code's command line tool isn't on this machine. Install it, then try again.",
            ));
        };
        let chosen_model = normalise(model, &MODEL_ALIASES);
        let chosen_effort = normalise(effort, &EFFORT_LEVELS);

        let mut command = CommandBuilder::new(&binary);
        if skip_permissions {
            command.arg("--dangerously-skip-permissions");
        }
        if let Some(model) = &chosen_model {
            command.args(["--model", model]);
        }
        if let Some(effort) = &chosen_effort {
            command.args(["--effort", effort]);
        }
        command.arg(mode.first_line(&launch.anchors));
        command.env_clear();
        for (key, value) in self.terminal_environment().await {
            command.env(key, value);
        }
        command.cwd(directory);

        let id = Id::random();
        let terminal = {
            let _gate = self.spawn_gate.lock().await;
            if self.live_map().len() >= self.max_live {
                return Err(RekallError::conflict(format!(
                    "Rekall is already running {} terminals. Close one before opening another.",
                    self.max_live
                )));
            }
            let terminal = self
                .spawn(id, launch.clone(), step_id, skip_permissions, chosen_model, chosen_effort, command)
                .map_err(|failed| RekallError::conflict(format!("Could not start claude: {failed}")))?;
            self.live_map().insert(id, terminal.clone());
            terminal
        };

        self.mark_running(task_id, step_id).await;
        info!("Opened terminal {id} to {mode} {} in {}", launch.anchors, launch.working_dir);
        Ok(terminal.view())
    }

    #[allow(clippy::too_many_arguments)]
    fn spawn(
        self: &Arc<Self>,
        id: Id,
        launch: TerminalLaunch,
        step_id: Option<Id>,
        skip_permissions: bool,
        model: Option<String>,
        effort: Option<String>,
        command: CommandBuilder,
    ) -> std::result::Result<Arc<Terminal>, String> {
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
            columns: AtomicU16::new(DEFAULT_COLUMNS),
            rows: AtomicU16::new(DEFAULT_ROWS),
            skip_permissions,
            model,
            effort,
            started_at: now,
            last_activity_at: Mutex::new(now),
            closing: AtomicBool::new(false),
            master: Mutex::new(pair.master),
            writer: Mutex::new(writer),
            killer: Mutex::new(killer),
            pid,
            exit,
            listeners: Mutex::new(Vec::new()),
            scrollback: Mutex::new(Scrollback::new(self.scrollback_bytes)),
        });

        let pumped = terminal.clone();
        std::thread::Builder::new()
            .name(format!("terminal-pty-{id}"))
            .spawn(move || pump(&pumped, reader))
            .map_err(|e| text(&e))?;

        let runtime = tokio::runtime::Handle::current();
        let manager = Arc::downgrade(self);
        std::thread::Builder::new().name(format!("terminal-wait-{id}")).spawn(move || {
            let code = match child.wait() {
                Ok(status) => status.exit_code() as i32,
                Err(_) => -1,
            };
            let _ = exit_sender.send(Some(code));
            if let Some(manager) = manager.upgrade() {
                runtime.spawn(async move { manager.on_exit(id).await });
            }
        })
        .map_err(|e| text(&e))?;
        Ok(terminal)
    }

    /// Move the running marker to another step without touching the process.
    async fn retarget(&self, terminal: &Arc<Terminal>, step_id: Option<Id>) -> TerminalView {
        let current = terminal.step_id();
        if let Some(step_id) = step_id {
            if Some(step_id) != current {
                self.release_running(terminal.launch.task_id, current).await;
                *terminal.step_id.lock().expect("never poisoned") = Some(step_id);
                self.mark_running(terminal.launch.task_id, Some(step_id)).await;
            }
        }
        *terminal.last_activity_at.lock().expect("never poisoned") = Instant::now();
        terminal.view()
    }

    async fn mark_running(&self, task_id: Id, step_id: Option<Id>) {
        let result = match step_id {
            Some(step_id) => self.task_steps.mark_running(Some(step_id)).await,
            None => self.task_review.session_running(task_id, true).await,
        };
        if let Err(failed) = result {
            warn!("Could not mark terminal work on task {task_id} running: {failed}");
        }
    }

    async fn release_running(&self, task_id: Id, step_id: Option<Id>) {
        let result = match step_id {
            Some(step_id) => self.task_steps.release_running(Some(step_id)).await,
            None => self.task_review.session_running(task_id, false).await,
        };
        if let Err(failed) = result {
            warn!("Could not release terminal work on task {task_id}: {failed}");
        }
    }

    /// Attach a pane: replay the scrollback, then follow live output. A closed terminal is a
    /// conflict. The backlog can land mid-redraw of a full-screen app, so the PTY is wobbled a
    /// row and back to make it repaint.
    pub fn attach(&self, id: Id, listener: Arc<dyn Listener>) -> Result<(TerminalView, u64)> {
        let terminal = self.require(id)?;
        // Subscribe before replaying: a chunk arriving in the gap is drawn twice, not dropped.
        let token = self.next_listener.fetch_add(1, Ordering::SeqCst);
        terminal.listeners.lock().expect("never poisoned").push((token, listener.clone()));
        let backlog = terminal.scrollback.lock().expect("never poisoned").snapshot();
        if !backlog.is_empty() {
            listener.output(&backlog);
            kick_resize(&terminal);
        }
        Ok((terminal.view(), token))
    }

    pub fn detach(&self, id: Id, token: u64) {
        if let Some(terminal) = self.live_map().get(&id) {
            terminal.listeners.lock().expect("never poisoned").retain(|(t, _)| *t != token);
        }
    }

    /// Feed raw bytes to the terminal's stdin. A dead PTY is a conflict.
    pub fn write(&self, id: Id, data: &[u8]) -> Result<()> {
        let terminal = self.require(id)?;
        {
            let mut writer = terminal.writer.lock().expect("never poisoned");
            if !terminal.alive() {
                return Err(RekallError::conflict("This terminal has closed."));
            }
            if writer.write_all(data).and_then(|_| writer.flush()).is_err() {
                return Err(RekallError::conflict("This terminal is no longer accepting input."));
            }
        }
        *terminal.last_activity_at.lock().expect("never poisoned") = Instant::now();
        Ok(())
    }

    /// Tell the PTY its new window size. Out-of-range values are clamped; a failure is logged.
    pub fn resize(&self, id: Id, columns: i64, rows: i64) {
        let Some(terminal) = self.live_map().get(&id).cloned() else { return };
        let columns = columns.clamp(1, MAX_DIMENSION) as u16;
        let rows = rows.clamp(1, MAX_DIMENSION) as u16;
        if terminal.apply_win_size(columns, rows) {
            terminal.columns.store(columns, Ordering::SeqCst);
            terminal.rows.store(rows, Ordering::SeqCst);
        }
    }

    /// Stop a terminal now: SIGTERM, a short grace, then SIGKILL. Idempotent.
    pub async fn close(&self, id: Id, reason: &str) -> Option<TerminalView> {
        let terminal = self.live_map().remove(&id)?;
        terminal.closing.store(true, Ordering::SeqCst);
        terminal.destroy();
        if !terminal.wait_for(Duration::from_secs(2)).await {
            terminal.destroy_forcibly();
        }
        terminal.notify_ended(terminal.safe_exit_value(), reason);
        self.release_running(terminal.launch.task_id, terminal.step_id()).await;
        self.announce_ended(&terminal);
        info!("Closed terminal {id} ({reason})");
        Some(terminal.view())
    }

    pub fn list(&self) -> Vec<TerminalView> {
        let mut views: Vec<TerminalView> = self.live_map().values().map(|t| t.view()).collect();
        views.sort_by_key(|v| v.started_at);
        views
    }

    pub fn get(&self, id: Id) -> Option<TerminalView> {
        self.live_map().get(&id).map(|t| t.view())
    }

    pub fn is_live(&self, id: Id) -> bool {
        self.live_map().get(&id).is_some_and(|t| t.alive())
    }

    pub fn live_count(&self) -> usize {
        self.live_map().len()
    }

    /// True while some live terminal is on this task, whoever opened it.
    pub fn has_live_terminal_for(&self, task_id: Id) -> bool {
        self.live_terminal_for_task(task_id).is_some_and(|t| t.alive())
    }

    /// Kill every PTY and close its socket as the application closes. Idempotent.
    pub async fn shutdown(&self) {
        self.stopped.store(true, Ordering::SeqCst);
        let terminals: Vec<Arc<Terminal>> = self.live_map().drain().map(|(_, t)| t).collect();
        for terminal in &terminals {
            terminal.closing.store(true, Ordering::SeqCst);
            terminal.destroy();
        }
        for terminal in &terminals {
            if !terminal.wait_for(Duration::from_secs(1)).await {
                terminal.destroy_forcibly();
            }
            let code = if terminal.alive() { 0 } else { terminal.safe_exit_value() };
            terminal.notify_ended(code, "The app was shutting down.");
            self.release_running(terminal.launch.task_id, terminal.step_id()).await;
            self.announce_ended(terminal);
        }
    }

    // ---------------------------------------------------------------- helpers

    async fn on_exit(&self, id: Id) {
        let Some(terminal) = self.live_map().remove(&id) else { return };
        let code = terminal.safe_exit_value();
        let detail = if code == 0 { "The terminal ended.".to_string() } else { format!("claude exited with code {code}") };
        terminal.notify_ended(code, &detail);
        self.release_running(terminal.launch.task_id, terminal.step_id()).await;
        self.announce_ended(&terminal);
        info!("Terminal {id} exited with code {code}");
    }

    fn announce_ended(&self, terminal: &Terminal) {
        let _ = self.ended.send(TerminalEnded { terminal_id: terminal.id, task_id: terminal.launch.task_id });
    }

    async fn sweep_idle(&self) {
        let cutoff = Instant::now().minus(TimeDelta::minutes(self.idle_minutes as i64));
        let idle: Vec<Id> = self
            .live_map()
            .values()
            .filter(|t| t.last_activity_at.lock().expect("never poisoned").is_before(&cutoff))
            .map(|t| t.id)
            .collect();
        for id in idle {
            self.close(id, &format!("Closed after {} minutes with no activity.", self.idle_minutes)).await;
        }
    }

    fn require(&self, id: Id) -> Result<Arc<Terminal>> {
        self.live_map()
            .get(&id)
            .cloned()
            .ok_or_else(|| RekallError::conflict("This terminal has closed. Open a new one to keep going."))
    }

    fn live_terminal_for_task(&self, task_id: Id) -> Option<Arc<Terminal>> {
        self.live_map().values().find(|t| t.launch.task_id == task_id).cloned()
    }

    /// The user's own shell environment, plus the `TERM`/`COLORTERM` a TUI needs.
    async fn terminal_environment(&self) -> HashMap<String, String> {
        let mut environment = self.cli.environment().await;
        environment.insert("TERM".into(), "xterm-256color".into());
        environment.insert("COLORTERM".into(), "truecolor".into());
        environment
    }
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
                debug!("PTY output for terminal {} closed: {ended}", terminal.id);
                break;
            }
        }
    }
}

/// Wobble the PTY down a row and back to its last known size: each step is a real size change,
/// so the kernel raises SIGWINCH both times and the TUI repaints in full.
fn kick_resize(terminal: &Terminal) {
    let rows = terminal.rows.load(Ordering::SeqCst);
    let columns = terminal.columns.load(Ordering::SeqCst);
    if rows <= 1 {
        return;
    }
    terminal.apply_win_size(columns, rows - 1);
    terminal.apply_win_size(columns, rows);
}

fn normalise(value: Option<&str>, allowed: &[&str]) -> Option<String> {
    let trimmed = rekall_common::jstr::strip(value?).to_lowercase();
    allowed.contains(&trimmed.as_str()).then_some(trimmed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_scrollback_keeps_the_newest_bytes() {
        let mut ring = Scrollback::new(4);
        ring.append(b"ab");
        assert_eq!(ring.snapshot(), b"ab");
        ring.append(b"cdef");
        assert_eq!(ring.snapshot(), b"cdef");
        ring.append(b"g");
        assert_eq!(ring.snapshot(), b"defg");
    }

    #[test]
    fn only_the_offered_model_and_effort_values_pass() {
        assert_eq!(normalise(Some(" Opus "), &MODEL_ALIASES).as_deref(), Some("opus"));
        assert_eq!(normalise(Some("gpt"), &MODEL_ALIASES), None);
        assert_eq!(normalise(None, &EFFORT_LEVELS), None);
    }
}
