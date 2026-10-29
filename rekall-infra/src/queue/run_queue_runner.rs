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

use async_trait::async_trait;
use rekall_kernel::{Instant, RekallError, Result};
use rekall_core::claude::TaskWorkService;
use rekall_core::step::TaskStepService;
use rekall_core::step::StepStreamEvent;
use rekall_core::review::TaskReviewEvent;
use rekall_core::{Clock, EventBus, PublishedEvent};
use tokio::sync::broadcast::{self, error::RecvError};
use tokio::sync::{mpsc, oneshot};

use super::run_queue_worker::{Command, Worker, WorkerParts};
use rekall_core::queue::{QueueRunner, RunQueueService, RunQueueView};
use crate::pty::{PtyTerminalManager, TerminalEnded};
use crate::usage::UsageReader;

/// The handle the rest of the application drives the queue through. Every call is a command to
/// the worker, which decides on one at a time.
#[derive(Clone)]
pub struct RunQueueRunner {
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
        let (commands, inbox) = mpsc::unbounded_channel();
        let ended = terminals.subscribe_ended();
        let parts = WorkerParts { queue, terminals, usage, work, steps, clock, settle_grace };
        tokio::spawn(Worker::new(parts, commands.clone()).run(inbox));
        spawn_clock(commands.clone(), tick);
        forward_step_and_review_moves(commands.clone(), events.subscribe());
        forward_session_ends(commands.clone(), ended);
        Self { commands }
    }

    /// Arm the queue and look at it at once rather than on the next tick.
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

    /// Stop deciding before the terminals go, as the application closes. The tick and the signal
    /// listeners end with the worker, on their next send.
    pub fn shutdown(&self) {
        let _ = self.commands.send(Command::Shutdown);
    }

    /// Take a look now: the console changed something the next tick would otherwise wait for.
    pub fn nudge(&self) {
        let _ = self.commands.send(Command::Tick);
    }
}

/// A tick every `tick`, the first one a full period after start. Ends once the worker is gone.
fn spawn_clock(commands: mpsc::UnboundedSender<Command>, tick: Duration) {
    tokio::spawn(async move {
        let every = tick.max(Duration::from_millis(10));
        let mut interval = tokio::time::interval_at(tokio::time::Instant::now() + every, every);
        loop {
            interval.tick().await;
            if commands.send(Command::Tick).is_err() {
                return;
            }
        }
    });
}

/// Every committed step or review move becomes a look at that task.
fn forward_step_and_review_moves(commands: mpsc::UnboundedSender<Command>, mut bus: broadcast::Receiver<PublishedEvent>) {
    tokio::spawn(async move {
        loop {
            let task_id = match bus.recv().await {
                Ok(event) => match task_moved_by(&event) {
                    Some(task_id) => task_id,
                    None => continue,
                },
                Err(RecvError::Lagged(_)) => continue,
                Err(RecvError::Closed) => return,
            };
            if commands.send(Command::Review(task_id)).is_err() {
                return;
            }
        }
    });
}

/// The task a committed step or review move belongs to; any other event moves nothing.
fn task_moved_by(event: &PublishedEvent) -> Option<rekall_kernel::Id> {
    if let Some(steps) = event.downcast::<StepStreamEvent>() {
        return Some(steps.task_id);
    }
    event.downcast::<TaskReviewEvent>().map(|review| review.task_id)
}

/// Every terminal that ends becomes a look at whether it was the one the runner had open.
fn forward_session_ends(commands: mpsc::UnboundedSender<Command>, mut ended: broadcast::Receiver<TerminalEnded>) {
    tokio::spawn(async move {
        loop {
            let terminal_id = match ended.recv().await {
                Ok(event) => event.terminal_id,
                Err(RecvError::Lagged(_)) => continue,
                Err(RecvError::Closed) => return,
            };
            if commands.send(Command::SessionEnded(terminal_id)).is_err() {
                return;
            }
        }
    });
}

#[async_trait]
impl QueueRunner for RunQueueRunner {
    fn nudge(&self) {
        RunQueueRunner::nudge(self)
    }

    async fn start(&self, queue: &RunQueueService, start_at: Option<Instant>) -> Result<RunQueueView> {
        let started = queue.start(start_at).await?;
        RunQueueRunner::nudge(self);
        Ok(started)
    }

    async fn stop(&self) -> Result<RunQueueView> {
        RunQueueRunner::stop(self).await
    }
}
