use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use rekall_core::chat::{ChatBody, ChatEvent, ChatListener};
use rekall_core::claude::{SessionKind, TerminalConnectionView, TerminalLaunch, TerminalView};
use rekall_kernel::{Id, Instant, RekallError, Result};

use super::chat_process::ChatProcess;
use super::event_journal::EventJournal;
use crate::session::ClaudeOptions;

/// One live chat session: its process, the task and step it works, and the journal its clients follow.
pub(super) struct ChatSession {
    pub(super) id: Id,
    pub(super) launch: TerminalLaunch,
    pub(super) step_id: Mutex<Option<Id>>,
    /// Whether it works a task's steps, the only chat session a new session can follow.
    pub(super) follows_steps: bool,
    pub(super) skip_permissions: bool,
    pub(super) model: Option<String>,
    pub(super) effort: Option<String>,
    pub(super) started_at: Instant,
    pub(super) last_activity_at: Mutex<Instant>,
    pub(super) closing: AtomicBool,
    pub(super) process: ChatProcess,
    /// Prompts `claude` is waiting on: each is answered once, and none outlives its turn.
    pending_permissions: Mutex<HashSet<String>>,
    journal: Mutex<EventJournal>,
}

impl ChatSession {
    pub(super) fn new(
        id: Id,
        launch: TerminalLaunch,
        step_id: Option<Id>,
        follows_steps: bool,
        options: ClaudeOptions,
        process: ChatProcess,
        backlog_bytes: usize,
    ) -> Self {
        let now = Instant::now();
        Self {
            id,
            launch,
            step_id: Mutex::new(step_id),
            follows_steps,
            skip_permissions: options.skip_permissions,
            model: options.model,
            effort: options.effort,
            started_at: now,
            last_activity_at: Mutex::new(now),
            closing: AtomicBool::new(false),
            process,
            pending_permissions: Mutex::new(HashSet::new()),
            journal: Mutex::new(EventJournal::new(backlog_bytes)),
        }
    }

    pub(super) fn alive(&self) -> bool {
        self.process.alive()
    }

    pub(super) fn awaiting_permission(&self) -> bool {
        !self.pending().is_empty()
    }

    pub(super) fn step_id(&self) -> Option<Id> {
        *self.step_id.lock().expect("never poisoned")
    }

    pub(super) fn view(&self) -> TerminalView {
        TerminalView {
            id: self.id,
            project_id: self.launch.project_id,
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
            kind: SessionKind::Chat,
            live: self.alive() && !self.closing.load(Ordering::SeqCst),
            awaiting_permission: self.awaiting_permission(),
            started_at: self.started_at,
            last_activity_at: *self.last_activity_at.lock().expect("never poisoned"),
            connections: self.launch.connections.iter().map(TerminalConnectionView::of).collect(),
        }
    }

    pub(super) fn touch(&self) {
        *self.last_activity_at.lock().expect("never poisoned") = Instant::now();
    }

    pub(super) fn publish(&self, body: ChatBody) -> ChatEvent {
        match &body {
            ChatBody::PermissionRequested { request_id, .. } => {
                self.pending().insert(request_id.clone());
            }
            ChatBody::TurnFinished { .. } => self.deny_pending_permissions(),
            _ => {}
        }
        self.touch();
        self.journal().publish(body)
    }

    /// Passes the person's answer to the prompt `request_id`. A prompt that is not waiting, because
    /// it was answered or its turn ended, is not found.
    pub(super) fn answer_permission(&self, request_id: &str, allow: bool) -> Result<()> {
        if !self.pending().remove(request_id) {
            return Err(RekallError::not_found("permission request", request_id));
        }
        self.process.answer_permission(request_id, allow)?;
        self.publish(ChatBody::PermissionResolved { request_id: request_id.to_string(), allowed: allow });
        Ok(())
    }

    /// Writes a user message to the process and shows it in the stream.
    pub(super) fn say(&self, text: &str) -> Result<()> {
        self.process.send(text)?;
        self.publish(ChatBody::UserMessage { text: text.to_string() });
        Ok(())
    }

    pub(super) fn attach(&self, token: u64, listener: Arc<dyn ChatListener>, after_seq: u64) -> Vec<ChatEvent> {
        self.journal().attach(token, listener, after_seq)
    }

    pub(super) fn detach(&self, token: u64) {
        self.journal().detach(token);
    }

    /// Tells every client how the session ended and lets go of them.
    pub(super) fn end(&self, exit_code: i32, detail: &str) {
        self.deny_pending_permissions();
        self.publish(ChatBody::Ended { exit_code, detail: detail.to_string() });
        let listeners = self.journal().take_listeners();
        for listener in listeners {
            listener.ended(exit_code, detail);
        }
    }

    fn deny_pending_permissions(&self) {
        let waiting: Vec<String> = self.pending().drain().collect();
        for request_id in waiting {
            self.touch();
            self.journal().publish(ChatBody::PermissionResolved { request_id, allowed: false });
        }
    }

    fn pending(&self) -> MutexGuard<'_, HashSet<String>> {
        self.pending_permissions.lock().expect("never poisoned")
    }

    fn journal(&self) -> MutexGuard<'_, EventJournal> {
        self.journal.lock().expect("never poisoned")
    }
}
