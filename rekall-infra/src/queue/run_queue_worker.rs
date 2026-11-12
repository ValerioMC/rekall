//! The runner's decisions: up to `parallel_sessions` items at a time, each on a task of its own,
//! along the line the `RunQueueRunner` module describes. Every decision runs on the worker's task,
//! in the order the commands arrive, so its memory of the items in hand needs no lock.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use rekall_kernel::{Id, Result};
use rekall_store::{RunQueueItemState, RunQueueState};
use rekall_core::claude::{TaskWorkService, TerminalHost};
use rekall_core::step::TaskStepService;
use rekall_core::worktree::TaskWorktreeService;
use rekall_core::Clock;
use tokio::sync::{mpsc, oneshot};
use tracing::{info, warn};

use super::usage_ceiling::Verdict;
use rekall_core::queue::{NextTurn, RunQueueService, RunQueueView, Snapshot};
use rekall_core::claude::UsageReader;

mod launch;
mod renewal;
mod review;
mod settling;

/// An item a session is working now.
#[derive(Clone, Debug)]
struct Active {
    generation: u64,
    item_id: Id,
    task_id: Id,
    terminal_id: Id,
    settled_steps: usize,
    /// Whether the item's detail says the session waits on a permission prompt.
    waiting_for_permission: bool,
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
    terminals: Arc<dyn TerminalHost>,
    usage: Arc<dyn UsageReader>,
    work: TaskWorkService,
    steps: TaskStepService,
    worktrees: TaskWorktreeService,
    clock: Clock,
    settle_grace: Duration,
    active: Vec<Active>,
    /// Items that could not be started this run, passed over until the queue is armed again.
    parked: HashSet<Id>,
    generation: u64,
    commands: mpsc::UnboundedSender<Command>,
}

/// What the worker decides with, handed over once by `RunQueueRunner::start`.
pub(super) struct WorkerParts {
    pub queue: RunQueueService,
    pub terminals: Arc<dyn TerminalHost>,
    pub usage: Arc<dyn UsageReader>,
    pub work: TaskWorkService,
    pub steps: TaskStepService,
    pub worktrees: TaskWorktreeService,
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
            worktrees: parts.worktrees,
            clock: parts.clock,
            settle_grace: parts.settle_grace,
            active: Vec::new(),
            parked: HashSet::new(),
            generation: 0,
            commands,
        }
    }

    /// Recovers what a restart left running, then decides on each command until shutdown.
    pub(super) async fn run(mut self, mut inbox: mpsc::UnboundedReceiver<Command>) {
        guarded("recover", self.queue.recover_after_restart().await);
        guarded("worktrees", self.worktrees.reconcile().await);
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
                guarded("refresh", self.queue.refresh_for_task(task_id).await);
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
                for stopping in std::mem::take(&mut self.active) {
                    self.close_session(stopping, "Stopped from the console.").await;
                }
                self.parked.clear();
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
        for active in self.active.clone() {
            if !active.closing && !self.terminals.get(active.terminal_id).is_some_and(|session| session.live) {
                self.session_ended(active.terminal_id).await?;
            } else {
                self.review(active.task_id).await?;
                self.note_permission_wait(active.generation).await?;
            }
        }
        self.advance().await
    }

    /// Fill the free places with the next waiting items, skipping and failing past the ones that
    /// cannot run. With every place taken, or nothing that may start while sessions run, it waits
    /// for one of them to end.
    async fn advance(&mut self) -> Result<()> {
        loop {
            let snapshot = self.queue.snapshot().await?;
            if snapshot.state != RunQueueState::Running || self.active.len() >= places(&snapshot) {
                return Ok(());
            }
            let next = match self.queue.next_turn(&self.parked).await? {
                NextTurn::Ready(next) => next,
                NextTurn::Empty => {
                    if self.active.is_empty() {
                        info!("Run queue: every queued task has had its turn");
                        self.queue.finish().await?;
                    }
                    return Ok(());
                }
                NextTurn::Blocked(blocked) => {
                    if self.active.is_empty() {
                        info!(waiting = blocked.len(), "Run queue: stopping, every waiting task is held back");
                        self.queue.stop_blocked(&blocked).await?;
                        self.parked.clear();
                    }
                    return Ok(());
                }
            };
            let verdict = self.ceiling(&snapshot).await;
            if !verdict.clear {
                if self.active.is_empty() {
                    info!(resume_at = ?verdict.resume_at, reason = ?verdict.reason, "Run queue: holding");
                    self.queue.hold(verdict.resume_at, verdict.reason.as_deref()).await?;
                }
                return Ok(());
            }
            self.launch(next, &snapshot).await?;
        }
    }

    fn mark_closing(&mut self, generation: u64) {
        if let Some(active) = self.active_mut(generation) {
            active.closing = true;
        }
    }

    fn is_current(&self, generation: u64) -> bool {
        self.active.iter().any(|active| active.generation == generation)
    }

    fn active_of(&self, generation: u64) -> Option<Active> {
        self.active.iter().find(|active| active.generation == generation).cloned()
    }

    fn active_mut(&mut self, generation: u64) -> Option<&mut Active> {
        self.active.iter_mut().find(|active| active.generation == generation)
    }

    /// Stop remembering a session, taking it out of the places in use.
    fn forget(&mut self, generation: u64) -> Option<Active> {
        let at = self.active.iter().position(|active| active.generation == generation)?;
        Some(self.active.remove(at))
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

/// How many tasks the queue may work at once.
fn places(snapshot: &Snapshot) -> usize {
    usize::try_from(snapshot.parallel_sessions).unwrap_or(1).max(1)
}

/// One failed decision is logged and the next tick tries again; it never kills the runner.
fn guarded(what: &str, result: Result<()>) {
    if let Err(failed) = result {
        warn!(%failed, action = what, "Run queue: an action failed");
    }
}
