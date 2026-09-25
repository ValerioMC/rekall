//! Works through the run queue, one task at a time, each in a `claude` terminal of its own.
//!
//! Every decision runs on one task, so the runner's memory of the item in hand needs no lock: a
//! clock tick every `rekall.run-queue.tick-seconds`, and a nudge from every step move, review move
//! and terminal end on the task it is running. The line for each item:
//!
//! 1. Take the first waiting item. None left: the queue goes idle.
//! 2. Check the usage ceiling. At or over it: hold until the window resets.
//! 3. Nothing open on the task: skip it. A terminal someone else has open on it, a missing folder
//!    or CLI: fail it with the reason, and go on.
//! 4. Open a terminal on the task, which types `/rk` with its anchors.
//! 5. Every claim is a step boundary, and the ceiling is checked again there. Over it: the
//!    terminal closes, the item waits again at the head, and the queue holds.
//! 6. Nothing left open: the item is finished, its terminal closed, and the next one starts.
//! 7. The terminal ends while work is still open: the item fails, and the next one starts.
//!
//! A terminal is closed the settle grace after the claim that ends its turn, so the session can
//! print its closing lines. The runner never interrupts a step.

use std::sync::Arc;
use std::time::Duration;

use rekall_common::{Id, RekallError, Result};
use rekall_model::{RunQueueItemState, RunQueueState};
use rekall_service::claude::{Remaining, TaskWork, TaskWorkService};
use rekall_service::step::TaskStepService;
use rekall_service::{Clock, DomainEvent, EventBus};
use tokio::sync::{mpsc, oneshot};
use tracing::{info, warn};

use super::service::{RunQueueService, Snapshot};
use super::usage_ceiling::{self, Verdict};
use super::view::{RunQueueItemView, RunQueueView};
use crate::pty::PtyTerminalManager;
use crate::usage::UsageReader;

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

enum Command {
    Tick,
    Review(Id),
    SessionEnded(Id),
    Settle { generation: u64, outcome: RunQueueItemState, reason: String },
    Pause { generation: u64, verdict: Verdict },
    Stop(oneshot::Sender<Result<RunQueueView>>),
}

#[derive(Clone)]
pub struct RunQueueRunner {
    commands: mpsc::UnboundedSender<Command>,
}

struct Worker {
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

impl RunQueueRunner {
    /// Start deciding: recover what a restart left running, then tick, and listen for the step,
    /// review and terminal-end signals of the task in hand.
    #[allow(clippy::too_many_arguments)]
    pub fn start(
        queue: RunQueueService,
        terminals: Arc<PtyTerminalManager>,
        usage: Arc<dyn UsageReader>,
        work: TaskWorkService,
        steps: TaskStepService,
        events: &EventBus,
        clock: Clock,
        tick: Duration,
        settle_grace: Duration,
    ) -> Self {
        let (commands, mut inbox) = mpsc::unbounded_channel();
        let mut worker = Worker {
            queue: queue.clone(),
            terminals: terminals.clone(),
            usage,
            work,
            steps,
            clock,
            settle_grace,
            active: None,
            generation: 0,
            commands: commands.clone(),
        };
        tokio::spawn(async move {
            guarded("recover", worker.queue.recover_after_restart().await);
            while let Some(command) = inbox.recv().await {
                worker.handle(command).await;
            }
        });

        let ticks = commands.clone();
        tokio::spawn(async move {
            let every = tick.max(Duration::from_millis(10));
            let mut interval = tokio::time::interval_at(tokio::time::Instant::now() + every, every);
            loop {
                interval.tick().await;
                if ticks.send(Command::Tick).is_err() {
                    return;
                }
            }
        });

        // Step and review moves, once they commit.
        let signals = commands.clone();
        let mut bus = events.subscribe();
        tokio::spawn(async move {
            loop {
                match bus.recv().await {
                    Ok(DomainEvent::Steps(event)) => {
                        if signals.send(Command::Review(event.task_id)).is_err() {
                            return;
                        }
                    }
                    Ok(DomainEvent::TaskReview(event)) => {
                        if signals.send(Command::Review(event.task_id)).is_err() {
                            return;
                        }
                    }
                    Ok(_) => {}
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => return,
                }
            }
        });

        let ends = commands.clone();
        let mut ended = terminals.subscribe_ended();
        tokio::spawn(async move {
            loop {
                match ended.recv().await {
                    Ok(event) => {
                        if ends.send(Command::SessionEnded(event.terminal_id)).is_err() {
                            return;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => return,
                }
            }
        });

        Self { commands }
    }

    /// Arm the queue and look at it at once rather than on the next tick.
    pub async fn start_queue(&self, queue: &RunQueueService, start_at: Option<rekall_common::Instant>) -> Result<RunQueueView> {
        let started = queue.start(start_at).await?;
        self.nudge();
        Ok(started)
    }

    /// Disarm the queue, closing the session it has open. Done on the runner, and waited for.
    pub async fn stop(&self) -> Result<RunQueueView> {
        let (reply, answer) = oneshot::channel();
        if self.commands.send(Command::Stop(reply)).is_err() {
            return Err(RekallError::conflict("Interrupted while stopping the queue."));
        }
        match tokio::time::timeout(Duration::from_secs(30), answer).await {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => Err(RekallError::conflict("Interrupted while stopping the queue.")),
            Err(_) => Err(RekallError::conflict("The queue did not stop in time. Try again.")),
        }
    }

    /// Take a look now: the console changed something the next tick would otherwise wait for.
    pub fn nudge(&self) {
        let _ = self.commands.send(Command::Tick);
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
            Command::Pause { generation, verdict } => {
                let result = self.pause(generation, verdict).await;
                guarded("settle", result);
            }
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
                info!("Run queue: holding until {:?} ({:?})", verdict.resume_at, verdict.reason);
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
            .open(item.task_id, None, snapshot.skip_permissions, snapshot.model.as_deref(), snapshot.effort.as_deref())
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
        info!("Run queue: running {} in terminal {}", item.anchor, opened.id);
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
            }
        }
        Ok(())
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
        info!("Run queue: paused at the ceiling until {:?} ({:?})", verdict.resume_at, verdict.reason);
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
                    warn!("Run queue: releasing step {step_id} failed: {failed}");
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
        warn!("Run queue: {what} failed: {failed}");
    }
}
