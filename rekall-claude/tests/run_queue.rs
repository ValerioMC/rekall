//! The run queue's runner end to end against a stand-in `claude`: it works through the queue one
//! terminal at a time, skips what has nothing open, fails a session that ends unclaimed, holds at
//! the usage ceiling, and stops on request.

mod support;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use rekall_claude::queue::{RunQueueRunner, RunQueueService};
use rekall_claude::usage::{ClaudeUsageView, Limit, Severity, UsageReader};
use rekall_common::Instant;
use rekall_model::{RunQueueItemState, RunQueueState, TaskStepState};
use support::{eventually, world, World};

/// A usage reading a test sets.
struct FixedUsage(Mutex<ClaudeUsageView>);

#[async_trait::async_trait]
impl UsageReader for FixedUsage {
    async fn current(&self) -> ClaudeUsageView {
        self.0.lock().unwrap().clone()
    }

    async fn refresh(&self) -> ClaudeUsageView {
        self.current().await
    }
}

fn reading(session_percent: f64) -> ClaudeUsageView {
    ClaudeUsageView::ok(
        vec![Limit {
            key: "session".into(),
            label: "Current session".into(),
            percent: session_percent,
            severity: Severity::Normal,
            resets_at: Some(Instant::now().plus(chrono::TimeDelta::hours(2))),
        }],
        Instant::now(),
    )
}

struct Harness {
    queue: RunQueueService,
    runner: RunQueueRunner,
    terminals: Arc<rekall_claude::pty::PtyTerminalManager>,
}

fn harness(world: &World, usage: ClaudeUsageView) -> Harness {
    let queue = RunQueueService::new(world.services.ctx.clone());
    let terminals = world.terminals(8);
    let runner = RunQueueRunner::start(
        queue.clone(),
        terminals.clone(),
        Arc::new(FixedUsage(Mutex::new(usage))),
        world.services.task_work.clone(),
        world.services.steps.clone(),
        &world.events,
        rekall_service::system_clock(),
        Duration::from_millis(100),
        Duration::from_millis(50),
    );
    Harness { queue, runner, terminals }
}

async fn item_states(queue: &RunQueueService) -> Vec<(RunQueueItemState, Option<String>)> {
    queue.view().await.unwrap().items.into_iter().map(|item| (item.state, item.detail)).collect()
}

#[tokio::test]
async fn the_queue_runs_each_task_in_turn_and_moves_on_once_its_work_is_claimed() {
    let world = world().await;
    let project = world.project("alpha", Some(world.folder.path())).await;
    let busy = world.task(&project, "busy").await;
    let step = world.step(&busy, "First", 0, TaskStepState::Open).await;
    let accepted = world.task(&project, "accepted").await;
    world.step(&accepted, "Only", 0, TaskStepState::Done).await;
    let harness = harness(&world, reading(10.0));

    harness.queue.add(busy.id).await.unwrap();
    harness.queue.add(accepted.id).await.unwrap();
    let started = harness.runner.start_queue(&harness.queue, None).await.unwrap();
    assert_eq!(started.state, RunQueueState::Running);

    eventually("the first task running in a terminal", || async {
        harness.terminals.has_live_terminal_for(busy.id)
            && item_states(&harness.queue).await[0].0 == RunQueueItemState::Running
    })
    .await;
    // The queue's session works on the task itself, not a step, so the step stays open until claimed.
    assert_eq!(world.step_state(&busy, &step).await, TaskStepState::Open);

    world.services.steps.transition(Some("alpha"), "busy", Some("1"), TaskStepState::Claimed).await.unwrap();

    eventually("the queue to finish", || async { harness.queue.view().await.unwrap().state == RunQueueState::Idle }).await;
    assert_eq!(
        item_states(&harness.queue).await,
        vec![
            (RunQueueItemState::Finished, Some("All of it claimed and waiting for review.".into())),
            (RunQueueItemState::Skipped, Some("Nothing to run: all of it is already accepted.".into())),
        ]
    );
    assert!(!harness.terminals.has_live_terminal_for(busy.id), "the finished task's terminal is closed");
}

#[tokio::test]
async fn a_session_that_ends_before_claiming_its_work_fails_the_item() {
    let world = world().await;
    let project = world.project("alpha", Some(world.folder.path())).await;
    let task = world.task(&project, "one").await;
    world.step(&task, "First", 0, TaskStepState::Open).await;
    let harness = harness(&world, reading(10.0));

    harness.queue.add(task.id).await.unwrap();
    harness.runner.start_queue(&harness.queue, None).await.unwrap();
    eventually("the terminal", || async { harness.terminals.has_live_terminal_for(task.id) }).await;
    let terminal = harness.terminals.list().into_iter().next().unwrap();
    harness.terminals.write(terminal.id, b"exit\n").unwrap();

    eventually("the item to fail", || async {
        item_states(&harness.queue).await[0].0 == RunQueueItemState::Failed
    })
    .await;
    assert_eq!(item_states(&harness.queue).await[0].1.as_deref(), Some("The session ended before its work was claimed."));
    eventually("the queue to finish", || async { harness.queue.view().await.unwrap().state == RunQueueState::Idle }).await;
}

#[tokio::test]
async fn usage_at_the_ceiling_holds_the_queue_before_anything_starts() {
    let world = world().await;
    let project = world.project("alpha", Some(world.folder.path())).await;
    let task = world.task(&project, "one").await;
    let harness = harness(&world, reading(92.4));

    harness.queue.update_settings(Some(80), false, None, None).await.unwrap();
    harness.queue.add(task.id).await.unwrap();
    harness.runner.start_queue(&harness.queue, None).await.unwrap();

    eventually("the hold", || async { harness.queue.view().await.unwrap().state == RunQueueState::Holding }).await;
    let view = harness.queue.view().await.unwrap();
    assert_eq!(view.hold_reason.as_deref(), Some("Current session is at 92%, at or over the 80% ceiling."));
    assert!(view.hold_until.is_some());
    assert_eq!(view.items[0].state, RunQueueItemState::Queued);
    assert_eq!(harness.terminals.live_count(), 0);
}

#[tokio::test]
async fn stopping_closes_the_session_and_puts_the_item_back_at_the_head() {
    let world = world().await;
    let project = world.project("alpha", Some(world.folder.path())).await;
    let task = world.task(&project, "one").await;
    let harness = harness(&world, reading(10.0));

    harness.queue.add(task.id).await.unwrap();
    harness.runner.start_queue(&harness.queue, None).await.unwrap();
    eventually("the terminal", || async { harness.terminals.has_live_terminal_for(task.id) }).await;

    let stopped = harness.runner.stop().await.unwrap();
    assert_eq!(stopped.state, RunQueueState::Idle);
    assert_eq!(stopped.items[0].state, RunQueueItemState::Queued);
    assert_eq!(
        stopped.items[0].detail.as_deref(),
        Some("Stopped from the console; it picks up at the next open step.")
    );
    assert_eq!(harness.terminals.live_count(), 0);

    let refused = harness.runner.start_queue(&harness.queue, Some(Instant::now().minus(chrono::TimeDelta::hours(1)))).await;
    assert_eq!(refused.unwrap_err().message(), "That start time has already passed. Pick a later one, or start now.");
}
