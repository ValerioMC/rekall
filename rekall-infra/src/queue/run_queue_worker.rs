//! The runner's decisions: one item at a time, on a task of its own, along the line the
//! `RunQueueRunner` module describes. Every decision runs on the worker's task, in the order the
//! commands arrive, so its memory of the item in hand needs no lock.

use std::sync::Arc;
use std::time::Duration;

use rekall_kernel::{Id, RekallError, Result};
use rekall_store::{RunQueueItemState, RunQueueState};
use rekall_core::claude::{Remaining, TaskWork, TaskWorkService, TerminalTarget};
use rekall_core::step::TaskStepService;
use rekall_core::Clock;
use tokio::sync::{mpsc, oneshot};
use tracing::{info, warn};

use super::usage_ceiling::{self, Verdict};
use rekall_core::queue::{RunQueueItemView, RunQueueService, RunQueueView, Snapshot};
use crate::pty::PtyTerminalManager;
use rekall_core::claude::TerminalMode;
use rekall_core::claude::UsageReader;

/// The item a session is working now.
#[derive(Clone, Debug)]
struct Active {
    generation: u64,
    item_id: Id,
    task_id: Id,
    terminal_id: Id,
    settled_steps: usize,
    /// Set once the runner has decided to close this terminal, so a late event changes nothing.
    closing: bool,
}

pub(super) enum Command {
    Tick,
    Review(Id),
    SessionEnded(Id),
    Settle { generation: u64, outcome: RunQueueItemState, reason: String },
    /// The step claimed last was the end of its session: continue the item in a fresh one.
    Renew { generation: u64 },
    Pause { generation: u64, verdict: Verdict },
    Stop(oneshot::Sender<Result<RunQueueView>>),
    /// The application is closing: stop deciding. The terminals are closed by their manager.
    Shutdown,
}

pub(super) struct Worker {
    queue: RunQueueService,
    terminals: Arc<PtyTerminalManager>,
    usage: Arc<dyn UsageReader>,
    work: TaskWorkService,
    steps: TaskStepService,
    clock: Clock,
    settle_grace: Duration,
    active: Option<Active>,
    generation: u64,
    commands: mpsc::UnboundedSender<Command>,
}

/// What the worker decides with, handed over once by `RunQueueRunner::start`.
pub(super) struct WorkerParts {
    pub queue: RunQueueService,
    pub terminals: Arc<PtyTerminalManager>,
    pub usage: Arc<dyn UsageReader>,
    pub work: TaskWorkService,
    pub steps: TaskStepService,
    pub clock: Clock,
    pub settle_grace: Duration,
}

impl Worker {
    pub(super) fn new(parts: WorkerParts, commands: mpsc::UnboundedSender<Command>) -> Self {
        Self {
            queue: parts.queue,
            terminals: parts.terminals,
            usage: parts.usage,
            work: parts.work,
            steps: parts.steps,
            clock: parts.clock,
            settle_grace: parts.settle_grace,
            active: None,
            generation: 0,
            commands,
        }
    }

    /// Recovers what a restart left running, then decides on each command until shutdown.
    pub(super) async fn run(mut self, mut inbox: mpsc::UnboundedReceiver<Command>) {
        guarded("recover", self.queue.recover_after_restart().await);
        while let Some(command) = inbox.recv().await {
            if matches!(command, Command::Shutdown) {
                break;
            }
            self.handle(command).await;
        }
    }
}

impl Worker {
    async fn handle(&mut self, command: Command) {
        match command {
            Command::Tick => {
                let result = self.tick().await;
                guarded("tick", result);
            }
            Command::Review(task_id) => {
                let result = self.review(task_id).await;
                guarded("signal", result);
            }
            Command::SessionEnded(terminal_id) => {
                let result = self.session_ended(terminal_id).await;
                guarded("signal", result);
            }
            Command::Settle { generation, outcome, reason } => {
                let result = self.settle(generation, outcome, &reason).await;
                guarded("settle", result);
            }
            Command::Renew { generation } => {
                let result = self.renew(generation).await;
                guarded("renew", result);
            }
            Command::Pause { generation, verdict } => {
                let result = self.pause(generation, verdict).await;
                guarded("settle", result);
            }
            Command::Shutdown => {}
            Command::Stop(reply) => {
                if let Some(stopping) = self.active.take() {
                    self.close_session(stopping, "Stopped from the console.").await;
                }
                let stopped = self.queue.stop("Stopped from the console; it picks up at the next open step.").await;
                let _ = reply.send(stopped);
            }
        }
    }

    async fn tick(&mut self) -> Result<()> {
        let snapshot = self.queue.snapshot().await?;
        let now = (self.clock)();
        match snapshot.state {
            RunQueueState::Idle => return Ok(()),
            RunQueueState::Scheduled => {
                if snapshot.start_at.is_some_and(|at| now.is_before(&at)) {
                    return Ok(());
                }
                info!("Run queue: start time reached");
                self.queue.resume().await?;
            }
            RunQueueState::Holding => {
                if snapshot.hold_until.is_some_and(|until| now.is_before(&until)) {
                    return Ok(());
                }
                info!("Run queue: hold over, taking a fresh usage reading");
                self.usage.refresh().await;
                self.queue.resume().await?;
            }
            RunQueueState::Running => {}
        }
        if let Some(active) = self.active.clone() {
            if !active.closing && !self.terminals.is_live(active.terminal_id) {
                self.session_ended(active.terminal_id).await?;
            } else {
                self.review(active.task_id).await?;
            }
            return Ok(());
        }
        self.advance().await
    }

    /// Start the next waiting item, skipping and failing past the ones that cannot run.
    async fn advance(&mut self) -> Result<()> {
        while self.active.is_none() {
            let snapshot = self.queue.snapshot().await?;
            if snapshot.state != RunQueueState::Running {
                return Ok(());
            }
            let Some(next) = self.queue.next_waiting().await? else {
                info!("Run queue: every queued task has had its turn");
                self.queue.finish().await?;
                return Ok(());
            };
            let verdict = self.ceiling(&snapshot).await;
            if !verdict.clear {
                info!(resume_at = ?verdict.resume_at, reason = ?verdict.reason, "Run queue: holding");
                self.queue.hold(verdict.resume_at, verdict.reason.as_deref()).await?;
                return Ok(());
            }
            self.launch(next, &snapshot).await?;
        }
        Ok(())
    }

    async fn launch(&mut self, item: RunQueueItemView, snapshot: &Snapshot) -> Result<()> {
        let before = match self.work.read(item.task_id).await {
            Ok(before) => before,
            Err(RekallError::UnknownAnchor(_)) => {
                return self.queue.mark_item(item.id, RunQueueItemState::Failed, Some("The task no longer exists.")).await;
            }
            Err(other) => return Err(other),
        };
        if !before.has_work() {
            return self.queue.mark_item(item.id, RunQueueItemState::Skipped, Some(nothing_open(before.remaining))).await;
        }
        if self.terminals.has_live_terminal_for(item.task_id) {
            return self
                .queue
                .mark_item(
                    item.id,
                    RunQueueItemState::Failed,
                    Some("A terminal was already open on this task. Close it and queue the task again."),
                )
                .await;
        }
        let opened = match self
            .terminals
            .open(
                TerminalTarget::Task(item.task_id),
                None,
                snapshot.skip_permissions,
                snapshot.model.as_deref(),
                snapshot.effort.as_deref(),
                TerminalMode::Work,
            )
            .await
        {
            Ok(opened) => opened,
            Err(refused @ (RekallError::Conflict(_) | RekallError::IllegalArgument(_) | RekallError::UnknownAnchor(_))) => {
                return self.queue.mark_item(item.id, RunQueueItemState::Failed, Some(refused.message())).await;
            }
            Err(other) => return Err(other),
        };
        self.generation += 1;
        self.active = Some(Active {
            generation: self.generation,
            item_id: item.id,
            task_id: item.task_id,
            terminal_id: opened.id,
            settled_steps: before.settled_steps,
            closing: false,
        });
        self.queue.mark_item(item.id, RunQueueItemState::Running, None).await?;
        info!(anchor = %item.anchor, terminal_id = %opened.id, "Run queue: running");
        Ok(())
    }

    /// A step or the review line moved on the task in hand: finished, or a boundary to check.
    async fn review(&mut self, task_id: Id) -> Result<()> {
        let Some(current) = self.active.clone() else { return Ok(()) };
        if current.closing || current.task_id != task_id {
            return Ok(());
        }
        let now = match self.work.read(task_id).await {
            Ok(now) => now,
            Err(RekallError::UnknownAnchor(_)) => {
                self.active = None;
                self.close_session(current, "The task was deleted.").await;
                return self.advance().await;
            }
            Err(other) => return Err(other),
        };
        if !now.has_work() {
            self.mark_closing();
            self.later(Command::Settle {
                generation: current.generation,
                outcome: RunQueueItemState::Finished,
                reason: finished(now.remaining).into(),
            });
            return Ok(());
        }
        if now.settled_steps > current.settled_steps {
            if let Some(active) = self.active.as_mut() {
                active.settled_steps = now.settled_steps;
            }
            let snapshot = self.queue.snapshot().await?;
            let verdict = self.ceiling(&snapshot).await;
            if !verdict.clear {
                self.mark_closing();
                self.later(Command::Pause { generation: current.generation, verdict });
                return Ok(());
            }
        }
        if self.is_spent(current.terminal_id, &now) {
            self.later(Command::Renew { generation: current.generation });
        }
        Ok(())
    }

    fn is_spent(&self, terminal_id: Id, work: &TaskWork) -> bool {
        self.terminals.get(terminal_id).is_some_and(|terminal| work.session_is_spent(&terminal.started_at))
    }

    /// Close the terminal that ended its session and open another on the same item, which stays
    /// `Running`. Decided again here, after the grace: a session that went on to the next step
    /// by itself keeps its terminal.
    async fn renew(&mut self, generation: u64) -> Result<()> {
        if !self.is_current(generation) {
            return Ok(());
        }
        let current = self.active.clone().expect("checked above");
        if current.closing {
            return Ok(());
        }
        let work = self.work.read(current.task_id).await?;
        if !self.is_spent(current.terminal_id, &work) {
            return Ok(());
        }
        self.mark_closing();
        match self.terminals.renew_session(current.terminal_id).await {
            Ok(renewed) => {
                if let Some(active) = self.active.as_mut() {
                    active.terminal_id = renewed.id;
                    active.closing = false;
                }
                info!(terminal_id = %renewed.id, "Run queue: a new session opened on the next step");
                Ok(())
            }
            Err(failure) => {
                warn!(%failure, task_id = %current.task_id, "Run queue: the next session could not open");
                self.active = None;
                self.release_running_steps(current.task_id).await;
                self.queue
                    .mark_item(current.item_id, RunQueueItemState::Failed, Some("The next session could not be opened."))
                    .await?;
                self.advance().await
            }
        }
    }

    fn mark_closing(&mut self) {
        if let Some(active) = self.active.as_mut() {
            active.closing = true;
        }
    }

    fn is_current(&self, generation: u64) -> bool {
        self.active.as_ref().is_some_and(|active| active.generation == generation)
    }

    /// The item is done with; close its terminal, record the outcome, and go on to the next.
    async fn settle(&mut self, generation: u64, outcome: RunQueueItemState, reason: &str) -> Result<()> {
        if !self.is_current(generation) {
            return Ok(());
        }
        let current = self.active.take().expect("checked above");
        self.close_session(current.clone(), "The run queue moved on to the next task.").await;
        self.queue.mark_item(current.item_id, outcome, Some(reason)).await?;
        self.advance().await
    }

    /// At the ceiling on a step boundary: close the session, put the item back at the head, hold.
    async fn pause(&mut self, generation: u64, verdict: Verdict) -> Result<()> {
        if !self.is_current(generation) {
            return Ok(());
        }
        let current = self.active.take().expect("checked above");
        self.close_session(current.clone(), "The run queue reached its usage ceiling.").await;
        self.queue
            .mark_item(current.item_id, RunQueueItemState::Queued, Some("Paused at the usage ceiling; it picks up at the next open step."))
            .await?;
        self.queue.hold(verdict.resume_at, verdict.reason.as_deref()).await?;
        info!(resume_at = ?verdict.resume_at, reason = ?verdict.reason, "Run queue: paused at the ceiling");
        Ok(())
    }

    /// The terminal in hand went away without the runner closing it.
    async fn session_ended(&mut self, terminal_id: Id) -> Result<()> {
        let Some(current) = self.active.clone() else { return Ok(()) };
        if current.closing || current.terminal_id != terminal_id {
            return Ok(());
        }
        self.active = None;
        self.release_running_steps(current.task_id).await;
        let after = self.read_or_none(current.task_id).await;
        match after {
            Some(after) if !after.has_work() => {
                self.queue.mark_item(current.item_id, RunQueueItemState::Finished, Some(finished(after.remaining))).await?;
            }
            _ => {
                self.queue
                    .mark_item(current.item_id, RunQueueItemState::Failed, Some("The session ended before its work was claimed."))
                    .await?;
            }
        }
        self.advance().await
    }

    async fn ceiling(&self, snapshot: &Snapshot) -> Verdict {
        if snapshot.ceiling_percent.is_none() {
            return Verdict::go();
        }
        let reading = self.usage.current().await;
        usage_ceiling::check(Some(&reading), snapshot.ceiling_percent, snapshot.model.as_deref(), (self.clock)())
    }

    /// Close the item's terminal and put back any step its session had marked running: the next
    /// session has to find it open to pick it up.
    async fn close_session(&self, mut current: Active, reason: &str) {
        current.closing = true;
        self.terminals.close(current.terminal_id, reason).await;
        self.release_running_steps(current.task_id).await;
    }

    async fn release_running_steps(&self, task_id: Id) {
        if let Some(state) = self.read_or_none(task_id).await {
            for step_id in state.running_step_ids {
                if let Err(failed) = self.steps.release_running(Some(step_id)).await {
                    warn!(%failed, %step_id, "Run queue: releasing a step failed");
                }
            }
        }
    }

    async fn read_or_none(&self, task_id: Id) -> Option<TaskWork> {
        self.work.read(task_id).await.ok()
    }

    fn later(&self, command: Command) {
        let commands = self.commands.clone();
        let grace = self.settle_grace;
        tokio::spawn(async move {
            tokio::time::sleep(grace).await;
            let _ = commands.send(command);
        });
    }
}

fn nothing_open(remaining: Remaining) -> &'static str {
    if remaining == Remaining::Accepted {
        "Nothing to run: all of it is already accepted."
    } else {
        "Nothing to run: all of it is claimed and waiting for review."
    }
}

fn finished(remaining: Remaining) -> &'static str {
    if remaining == Remaining::Accepted {
        "All of it accepted."
    } else {
        "All of it claimed and waiting for review."
    }
}

/// One failed decision is logged and the next tick tries again; it never kills the runner.
fn guarded(what: &str, result: Result<()>) {
    if let Err(failed) = result {
        warn!(%failed, action = what, "Run queue: an action failed");
    }
}
