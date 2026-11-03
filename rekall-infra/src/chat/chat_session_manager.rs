//! Every live chat session: a headless `claude` opened on a task or project, followed by clients
//! through its event journal, written to, interrupted, closed, and swept when idle. It meets the
//! obligations a PTY terminal does: the running marker, the cap, the end announced however it came.
//! Nothing is persisted; strays are bounded by the cap, an idle sweep and a shutdown hook.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use std::time::Duration;

use async_trait::async_trait;
use chrono::TimeDelta;
use rekall_core::chat::{ChatAttachment, ChatBody, ChatListener, ChatPipe};
use rekall_core::claude::{TerminalEnded, TerminalHost, TerminalLaunch, TerminalLaunchService, TerminalMode, TerminalOpening, TerminalTarget, TerminalView};
use rekall_core::commit::StepSnapshotService;
use rekall_core::review::TaskReviewService;
use rekall_core::step::TaskStepService;
use rekall_core::telemetry::TelemetrySettingsService;
use rekall_kernel::{Id, Instant, RekallError, Result};
use tokio::process::Command;
use tokio::sync::{broadcast, mpsc};
use tracing::info;

use super::chat_command::chat_command;
use super::chat_process::spawn_chat_process;
use super::chat_session::ChatSession;
use crate::cli::ClaudeCli;
use crate::config::ClaudeConfig;
use crate::connection::CredentialBroker;
use crate::session::{require_folder, spawn_idle_sweep, ClaudeOptions, EndedBus, SessionLauncher, WorkMarker};

const CLOSE_GRACE: Duration = Duration::from_secs(2);
const SHUTDOWN_GRACE: Duration = Duration::from_secs(1);

pub struct ChatSessionManager {
    me: Weak<Self>,
    launch_service: TerminalLaunchService,
    launcher: SessionLauncher,
    marker: WorkMarker,
    ended: EndedBus,
    max_live: usize,
    idle_minutes: u64,
    backlog_bytes: usize,
    live: Mutex<HashMap<Id, Arc<ChatSession>>>,
    /// Serialises the cap check and registration so two fast opens cannot both slip past the cap.
    spawn_gate: tokio::sync::Mutex<()>,
    next_listener: AtomicU64,
    stopped: AtomicBool,
}

impl ChatSessionManager {
    /// The journal of each session keeps `config.scrollback_bytes` of serialised events, the way
    /// a terminal keeps that many bytes of output.
    pub fn new(
        launch_service: TerminalLaunchService,
        cli: ClaudeCli,
        task_steps: TaskStepService,
        task_review: TaskReviewService,
        step_snapshots: StepSnapshotService,
        broker: Arc<CredentialBroker>,
        telemetry: TelemetrySettingsService,
        config: &ClaudeConfig,
    ) -> Arc<Self> {
        Arc::new_cyclic(|me| Self {
            me: me.clone(),
            launch_service,
            launcher: SessionLauncher::new(cli, broker, telemetry),
            marker: WorkMarker::new(task_steps, task_review, step_snapshots),
            ended: EndedBus::new(),
            max_live: config.max_sessions,
            idle_minutes: config.idle_minutes,
            backlog_bytes: config.scrollback_bytes,
            live: Mutex::new(HashMap::new()),
            spawn_gate: tokio::sync::Mutex::new(()),
            next_listener: AtomicU64::new(0),
            stopped: AtomicBool::new(false),
        })
    }

    /// Sweep idle sessions every `sweep_minutes`.
    pub fn start_reaper(self: &Arc<Self>, sweep_minutes: u64) {
        let every = Duration::from_secs(sweep_minutes.max(1) * 60);
        spawn_idle_sweep(self, every, |manager: Arc<Self>| async move {
            if manager.stopped.load(Ordering::SeqCst) {
                return false;
            }
            manager.sweep_idle().await;
            true
        });
    }

    /// Closes every session with no activity for `idle-minutes`, except one waiting on a permission
    /// prompt: that one is waiting on a person, and closing it would fail the work it is on.
    pub async fn sweep_idle(&self) {
        let cutoff = Instant::now().minus(TimeDelta::minutes(self.idle_minutes as i64));
        let idle: Vec<Id> = self
            .live_map()
            .values()
            .filter(|session| !session.awaiting_permission())
            .filter(|session| session.last_activity_at.lock().expect("never poisoned").is_before(&cutoff))
            .map(|session| session.id)
            .collect();
        for id in idle {
            self.close_session(id, &format!("Closed after {} minutes with no activity.", self.idle_minutes)).await;
        }
    }

    /// Stops every session as the application closes. Idempotent.
    pub async fn shutdown(&self) {
        self.stopped.store(true, Ordering::SeqCst);
        let sessions: Vec<Arc<ChatSession>> = self.live_map().drain().map(|(_, session)| session).collect();
        for session in &sessions {
            session.closing.store(true, Ordering::SeqCst);
            session.process.terminate();
        }
        for session in &sessions {
            if !session.process.wait_for(SHUTDOWN_GRACE).await {
                session.process.kill();
                session.process.wait_for(SHUTDOWN_GRACE).await;
            }
            self.finish(session, "The app was shutting down.").await;
        }
    }

    pub fn live_count(&self) -> usize {
        self.live_map().len()
    }

    fn live_map(&self) -> MutexGuard<'_, HashMap<Id, Arc<ChatSession>>> {
        self.live.lock().expect("never poisoned")
    }

    fn require(&self, id: Id) -> Result<Arc<ChatSession>> {
        self.live_map()
            .get(&id)
            .cloned()
            .ok_or_else(|| RekallError::conflict("This chat session has closed. Open a new one to keep going."))
    }

    fn live_session_on(&self, target: TerminalTarget) -> Option<Arc<ChatSession>> {
        self.live_map().values().find(|session| session.launch.target() == target).cloned()
    }

    /// Starts the process and records it as live, under the cap. The check and the record are
    /// one step behind `spawn_gate`, so two fast opens cannot both slip past the cap.
    #[allow(clippy::too_many_arguments)]
    async fn register(
        &self,
        id: Id,
        launch: TerminalLaunch,
        step_id: Option<Id>,
        mode: TerminalMode,
        options: ClaudeOptions,
        command: Command,
        first_line: &str,
    ) -> Result<Arc<ChatSession>> {
        let _gate = self.spawn_gate.lock().await;
        if self.live_map().len() >= self.max_live {
            return Err(RekallError::conflict(format!(
                "Rekall is already running {} chat sessions. Close one before opening another.",
                self.max_live
            )));
        }
        let (process, events) = spawn_chat_process(id, command, first_line)?;
        let session = Arc::new(ChatSession::new(id, launch, step_id, mode, options, process, self.backlog_bytes));
        session.publish(ChatBody::UserMessage { text: first_line.to_string() });
        self.live_map().insert(id, session.clone());
        tokio::spawn(follow_process(self.me.clone(), session.clone(), events));
        Ok(session)
    }

    /// Move the running marker to another step without touching the process.
    async fn retarget(&self, session: &Arc<ChatSession>, step_id: Option<Id>) -> TerminalView {
        let current = session.step_id();
        if let Some(step_id) = step_id {
            if Some(step_id) != current {
                self.marker.release_running(session.launch.task_id, current).await;
                *session.step_id.lock().expect("never poisoned") = Some(step_id);
                self.marker.mark_running(session.launch.task_id, Some(step_id)).await;
            }
        }
        session.touch();
        session.view()
    }

    async fn close_session(&self, id: Id, reason: &str) -> Option<TerminalView> {
        let session = self.live_map().remove(&id)?;
        session.closing.store(true, Ordering::SeqCst);
        session.process.terminate();
        if !session.process.wait_for(CLOSE_GRACE).await {
            session.process.kill();
            session.process.wait_for(CLOSE_GRACE).await;
        }
        self.finish(&session, reason).await;
        info!(terminal_id = %id, %reason, "Closed chat session");
        Some(session.view())
    }

    async fn on_exit(&self, id: Id) {
        let Some(session) = self.live_map().remove(&id) else { return };
        let code = session.process.exit_code().unwrap_or(-1);
        let detail = if code == 0 { "The chat session ended.".to_string() } else { format!("claude exited with code {code}") };
        self.finish(&session, &detail).await;
        info!(terminal_id = %id, code, "Chat session exited");
    }

    async fn finish(&self, session: &Arc<ChatSession>, detail: &str) {
        session.end(session.process.exit_code().unwrap_or(-1), detail);
        self.launcher.release(session.id);
        if session.mode.follows_steps() {
            self.marker.release_running(session.launch.task_id, session.step_id()).await;
        }
        self.ended.announce(session.id, session.launch.task_id);
    }
}

/// Feeds the process's events to its journal, then ends the session once the process is gone.
async fn follow_process(manager: Weak<ChatSessionManager>, session: Arc<ChatSession>, mut events: mpsc::Receiver<ChatBody>) {
    while let Some(body) = events.recv().await {
        session.publish(body);
    }
    if !session.process.wait_for(CLOSE_GRACE).await {
        session.process.kill();
        session.process.wait_for(CLOSE_GRACE).await;
    }
    if let Some(manager) = manager.upgrade() {
        manager.on_exit(session.id).await;
    }
}

#[async_trait]
impl TerminalHost for ChatSessionManager {
    /// Start a headless `claude` for this target in its project's folder, with the `/rk` line of the
    /// mode sent first. A missing folder or CLI is a retriable conflict. Only a work opening is on a
    /// step: the step is marked `RUNNING` while the session is on it, or the review line on a task
    /// with no checklist. A second open on a target that already has a session reuses it: a work open
    /// is retargeted, and every mode has its line sent into that session.
    async fn open(&self, opening: TerminalOpening) -> Result<TerminalView> {
        let TerminalOpening { target, step_id, skip_permissions, model, effort, mode, .. } = opening;
        let launch = self.launch_service.resolve(target).await?;
        let follows_steps = mode.follows_steps();
        let step_id = if follows_steps { step_id } else { None };

        if let Some(existing) = self.live_session_on(target) {
            let view = if follows_steps { self.retarget(&existing, step_id).await } else { existing.view() };
            existing.say(&mode.first_line(&launch.anchors))?;
            return Ok(view);
        }

        let directory = require_folder(&launch)?;
        let binary = self.launcher.require_cli().await?;
        let mut options = ClaudeOptions::chosen(skip_permissions, model.as_deref(), effort.as_deref());
        let id = Id::random();
        let environment = self.launcher.environment(id, &launch, &mut options).await?;
        let command = chat_command(&binary, &options, environment, directory);
        let first_line = mode.first_line(&launch.anchors);

        let session = match self.register(id, launch.clone(), step_id, mode.clone(), options, command, &first_line).await {
            Ok(session) => session,
            Err(refused) => {
                self.launcher.release(id);
                return Err(refused);
            }
        };

        if follows_steps {
            self.marker.mark_running(launch.task_id, step_id).await;
        }
        info!(terminal_id = %id, %mode, anchors = %launch.anchors, working_dir = %launch.working_dir, "Opened chat session");
        Ok(session.view())
    }

    fn list(&self) -> Vec<TerminalView> {
        let mut views: Vec<TerminalView> = self.live_map().values().map(|session| session.view()).collect();
        views.sort_by_key(|view| view.started_at);
        views
    }

    fn get(&self, id: Id) -> Option<TerminalView> {
        self.live_map().get(&id).map(|session| session.view())
    }

    async fn close(&self, id: Id, reason: &str) -> Option<TerminalView> {
        self.close_session(id, reason).await
    }

    fn subscribe_ended(&self) -> broadcast::Receiver<TerminalEnded> {
        self.ended.subscribe()
    }

    fn has_live_for(&self, task_id: Id) -> bool {
        self.live_session_on(TerminalTarget::Task(task_id)).is_some_and(|session| session.alive())
    }

    fn work_session_for(&self, task_id: Id) -> Option<TerminalView> {
        self.live_session_on(TerminalTarget::Task(task_id))
            .filter(|session| session.alive() && session.mode.follows_steps())
            .map(|session| session.view())
    }

    async fn renew(&self, id: Id) -> Result<TerminalView> {
        let session = self.require(id)?;
        if !session.mode.follows_steps() {
            return Err(RekallError::illegal("Only a chat session working a task's steps starts a new session."));
        }
        let view = session.view();
        self.close_session(id, "The next step opens a new session.").await;
        self.open(TerminalOpening {
            target: session.launch.target(),
            step_id: None,
            skip_permissions: view.skip_permissions,
            model: view.model,
            effort: view.effort,
            mode: TerminalMode::Work,
            kind: view.kind,
        })
        .await
    }
}

impl ChatPipe for ChatSessionManager {
    /// Replays the journal after `after_seq`, then follows live. A closed session is a conflict.
    fn attach(&self, id: Id, listener: Arc<dyn ChatListener>, after_seq: u64) -> Result<ChatAttachment> {
        let session = self.require(id)?;
        let token = self.next_listener.fetch_add(1, Ordering::SeqCst);
        let backlog = session.attach(token, listener, after_seq);
        Ok(ChatAttachment { view: session.view(), backlog, token })
    }

    fn detach(&self, id: Id, token: u64) {
        if let Some(session) = self.live_map().get(&id) {
            session.detach(token);
        }
    }

    fn send(&self, id: Id, text: &str) -> Result<()> {
        self.require(id)?.say(text)
    }

    fn interrupt(&self, id: Id) -> Result<()> {
        let session = self.require(id)?;
        session.process.interrupt()?;
        session.touch();
        Ok(())
    }

    fn answer_permission(&self, id: Id, request_id: &str, allow: bool) -> Result<()> {
        self.require(id)?.answer_permission(request_id, allow)
    }
}
