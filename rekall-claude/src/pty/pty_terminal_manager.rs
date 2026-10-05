//! Every live terminal: opened on a task, attached to by panes, written to, resized, closed, and
//! swept when idle. The process inside each is `pty_process`'s; the command it runs is
//! `claude_command`'s.

use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::TimeDelta;
use portable_pty::CommandBuilder;
use rekall_common::{Id, Instant, RekallError, Result};
use rekall_service::claude::{TerminalLaunch, TerminalLaunchService};
use rekall_service::review::TaskReviewService;
use rekall_service::step::TaskStepService;
use tokio::sync::broadcast;
use tracing::{info, warn};

use crate::cli::ClaudeCli;
use crate::config::ClaudeConfig;
use crate::terminal::TerminalMode;

use super::claude_command::{claude_command, ClaudeOptions};
use super::pty_process::{kick_resize, spawn_terminal};
use super::terminal::Terminal;
use super::{Listener, TerminalEnded, TerminalView};

const MAX_DIMENSION: i64 = 1000;

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
    /// `RUNNING` while the terminal is on it; none means the task itself. A plan or a generate is
    /// always on the task, so every mode but [`TerminalMode::Work`] drops the step id. A second
    /// open on a task that already has a terminal reuses it: a work open is retargeted, and every
    /// mode has its line typed into that session so it reloads the task as it stands.
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
        let step_id = if mode.follows_steps() { step_id } else { None };

        if let Some(existing) = self.live_terminal_for_task(task_id) {
            let view = if mode.follows_steps() { self.retarget(&existing, step_id).await } else { existing.view() };
            self.write(existing.id, format!("{}\r", mode.first_line(&launch.anchors)).as_bytes())?;
            return Ok(view);
        }

        let directory = require_folder(&launch)?;
        let binary = self.require_cli().await?;
        let options = ClaudeOptions::chosen(skip_permissions, model, effort);
        let environment = self.cli.environment().await;
        let command = claude_command(&binary, &options, &mode.first_line(&launch.anchors), environment, directory);

        let id = Id::random();
        let terminal = self.register(id, launch.clone(), step_id, options, command).await?;

        self.mark_running(task_id, step_id).await;
        info!("Opened terminal {id} to {mode} {} in {}", launch.anchors, launch.working_dir);
        Ok(terminal.view())
    }

    /// Starts the process and records it as live, under the cap. The check and the record are
    /// one step behind `spawn_gate`, so two fast opens cannot both slip past the cap.
    async fn register(
        self: &Arc<Self>,
        id: Id,
        launch: TerminalLaunch,
        step_id: Option<Id>,
        options: ClaudeOptions,
        command: CommandBuilder,
    ) -> Result<Arc<Terminal>> {
        let _gate = self.spawn_gate.lock().await;
        if self.live_map().len() >= self.max_live {
            return Err(RekallError::conflict(format!(
                "Rekall is already running {} terminals. Close one before opening another.",
                self.max_live
            )));
        }
        let runtime = tokio::runtime::Handle::current();
        let manager = Arc::downgrade(self);
        let on_exit = move || {
            if let Some(manager) = manager.upgrade() {
                runtime.spawn(async move { manager.on_exit(id).await });
            }
        };
        let terminal = spawn_terminal(id, launch, step_id, options, command, self.scrollback_bytes, on_exit)
            .map_err(|failed| RekallError::conflict(format!("Could not start claude: {failed}")))?;
        self.live_map().insert(id, terminal.clone());
        Ok(terminal)
    }

    async fn require_cli(&self) -> Result<PathBuf> {
        self.cli.locate().await.ok_or_else(|| {
            RekallError::conflict("Claude Code's command line tool isn't on this machine. Install it, then try again.")
        })
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
    /// conflict. The backlog can land mid-redraw of a full-screen app, and a pane switching to a
    /// live session sends the size the PTY already has, so the PTY is wobbled a row and back.
    pub fn attach(&self, id: Id, listener: Arc<dyn Listener>) -> Result<(TerminalView, u64)> {
        let terminal = self.require(id)?;
        // Subscribe before replaying: a chunk arriving in the gap is drawn twice, not dropped.
        let token = self.next_listener.fetch_add(1, Ordering::SeqCst);
        terminal.listeners.lock().expect("never poisoned").push((token, listener.clone()));
        let backlog = terminal.scrollback.lock().expect("never poisoned").snapshot();
        if !backlog.is_empty() {
            listener.output(&backlog);
            kick_resize(terminal.clone());
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
    /// The size it already has is skipped: it raises no SIGWINCH, and mid-kick it would cut the
    /// shrink short before claude saw it.
    pub fn resize(&self, id: Id, columns: i64, rows: i64) {
        let Some(terminal) = self.live_map().get(&id).cloned() else { return };
        let columns = columns.clamp(1, MAX_DIMENSION) as u16;
        let rows = rows.clamp(1, MAX_DIMENSION) as u16;
        let mut size = terminal.size.lock().expect("never poisoned");
        if *size == (columns, rows) {
            return;
        }
        if terminal.apply_win_size(columns, rows) {
            *size = (columns, rows);
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
}

/// The project's folder, which has to be there for a session to start in it.
fn require_folder(launch: &TerminalLaunch) -> Result<&Path> {
    let directory = Path::new(&launch.working_dir);
    if !directory.is_dir() {
        return Err(RekallError::conflict(format!(
            "This project's folder ({}) is not there. Set it again on the project page.",
            launch.working_dir
        )));
    }
    Ok(directory)
}

#[cfg(test)]
#[path = "../../tests/unit/pty/pty_terminal_manager_tests.rs"]
mod tests;
