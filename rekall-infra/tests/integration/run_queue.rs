//! The run queue's runner end to end against a stand-in `claude`: it works through the queue one
//! terminal at a time, skips what has nothing open, fails a session that ends unclaimed, holds at
//! the usage ceiling, and stops on request.

mod support;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use rekall_core::queue::{QueueRunner, RunQueueFrame, RunQueueService};
use rekall_core::claude::StepProgressState;
use rekall_infra::queue::RunQueueRunner;
use rekall_core::claude::{ClaudeUsageView, Limit, SessionKind, Severity, UsageReader};
use rekall_kernel::Instant;
use rekall_core::flow::RouteTarget;
use rekall_store::{RunQueueItemState, RunQueueState, TaskStepState};
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
    terminals: Arc<rekall_infra::pty::PtyTerminalManager>,
}

fn harness(world: &World, usage: ClaudeUsageView) -> Harness {
    let queue = RunQueueService::new(world.services.ctx.clone());
    let terminals = world.terminals(8);
    let runner = RunQueueRunner::start(
        queue.clone(),
        Arc::new(rekall_infra::pty::PtyTerminals::new(terminals.clone())),
        Arc::new(FixedUsage(Mutex::new(usage))),
        world.services.task_work.clone(),
        world.services.steps.clone(),
        world.worktrees().0,
        &world.events,
        rekall_core::system_clock(),
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
    let started = harness.runner.start(&harness.queue, None).await.unwrap();
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
    harness.runner.start(&harness.queue, None).await.unwrap();
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

    harness.queue.update_settings(Some(80), false, None, None, SessionKind::Terminal, 1).await.unwrap();
    harness.queue.add(task.id).await.unwrap();
    harness.runner.start(&harness.queue, None).await.unwrap();

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
    harness.runner.start(&harness.queue, None).await.unwrap();
    eventually("the terminal", || async { harness.terminals.has_live_terminal_for(task.id) }).await;

    let stopped = harness.runner.stop().await.unwrap();
    assert_eq!(stopped.state, RunQueueState::Idle);
    assert_eq!(stopped.items[0].state, RunQueueItemState::Queued);
    assert_eq!(
        stopped.items[0].detail.as_deref(),
        Some("Stopped from the console; it picks up at the next open step.")
    );
    assert_eq!(harness.terminals.live_count(), 0);

    let refused = harness.runner.start(&harness.queue, Some(Instant::now().minus(chrono::TimeDelta::hours(1)))).await;
    assert_eq!(refused.unwrap_err().message(), "That start time has already passed. Pick a later one, or start now.");
}

async fn claim(world: &World, task: &str, step: &str) {
    world.services.steps.transition(Some("alpha"), task, Some(step), TaskStepState::Claimed).await.unwrap();
}

#[tokio::test]
async fn claiming_every_step_of_a_checklist_moves_the_queue_to_the_next_task_with_nothing_accepted() {
    let world = world().await;
    let project = world.project("alpha", Some(world.folder.path())).await;
    let first = world.task(&project, "first").await;
    for (position, title) in ["One", "Two", "Three"].into_iter().enumerate() {
        world.step(&first, title, position as i32, TaskStepState::Open).await;
    }
    let second = world.task(&project, "second").await;
    world.step(&second, "Only", 0, TaskStepState::Open).await;
    let harness = harness(&world, reading(10.0));
    harness.queue.add(first.id).await.unwrap();
    harness.queue.add(second.id).await.unwrap();
    harness.runner.start(&harness.queue, None).await.unwrap();
    eventually("the first task running", || async { harness.terminals.has_live_terminal_for(first.id) }).await;

    claim(&world, "first", "One").await;
    claim(&world, "first", "Two").await;
    assert_eq!(item_states(&harness.queue).await[0].0, RunQueueItemState::Running, "open work keeps it running");
    claim(&world, "first", "Three").await;

    eventually("the second task running", || async { harness.terminals.has_live_terminal_for(second.id) }).await;
    let states = item_states(&harness.queue).await;
    assert_eq!(states[0], (RunQueueItemState::Finished, Some("All of it claimed and waiting for review.".into())));
    assert_eq!(states[1].0, RunQueueItemState::Running);
}

#[tokio::test]
async fn a_flow_that_reaches_its_end_finishes_though_steps_were_never_run_and_the_queue_moves_on() {
    let world = world().await;
    let project = world.project("alpha", Some(world.folder.path())).await;
    let flow = world.task(&project, "flow").await;
    let check = world.step(&flow, "Check", 0, TaskStepState::Open).await;
    let fix = world.step(&flow, "Fix", 1, TaskStepState::Open).await;
    world.step(&flow, "Report", 2, TaskStepState::Open).await;
    world.services.step_links.add(check.id, RouteTarget::End, Some("passed")).await.unwrap();
    world.services.step_links.add(check.id, RouteTarget::Step(fix.id), Some("failed")).await.unwrap();
    let after = world.task(&project, "after").await;
    world.step(&after, "Only", 0, TaskStepState::Open).await;
    let harness = harness(&world, reading(10.0));
    harness.queue.add(flow.id).await.unwrap();
    harness.queue.add(after.id).await.unwrap();
    harness.runner.start(&harness.queue, None).await.unwrap();
    eventually("the flow running", || async { harness.terminals.has_live_terminal_for(flow.id) }).await;

    world.services.flow.transition(Some("alpha"), "flow", Some("Check"), TaskStepState::Claimed, Some("passed"), None).await.unwrap();

    eventually("the next task running", || async { harness.terminals.has_live_terminal_for(after.id) }).await;
    assert_eq!(
        item_states(&harness.queue).await[0],
        (RunQueueItemState::Finished, Some("Flow finished: the steps it did not need were left unrun.".into()))
    );
    assert!(!harness.terminals.has_live_terminal_for(flow.id));
}

#[tokio::test]
async fn a_task_with_claimed_steps_and_open_ones_is_worked_for_the_open_ones_only() {
    let world = world().await;
    let project = world.project("alpha", Some(world.folder.path())).await;
    let task = world.task(&project, "partial").await;
    let done = world.step(&task, "Earlier", 0, TaskStepState::Claimed).await;
    let later = world.step(&task, "Later", 1, TaskStepState::Open).await;
    let harness = harness(&world, reading(10.0));
    harness.queue.add(task.id).await.unwrap();
    harness.runner.start(&harness.queue, None).await.unwrap();

    eventually("the task running", || async { harness.terminals.has_live_terminal_for(task.id) }).await;
    assert_eq!(world.step_state(&task, &done).await, TaskStepState::Claimed);

    claim(&world, "partial", "Later").await;
    eventually("the queue to finish", || async { harness.queue.view().await.unwrap().state == RunQueueState::Idle }).await;
    assert_eq!(world.step_state(&task, &later).await, TaskStepState::Claimed);
}

#[tokio::test]
async fn a_step_a_session_left_running_goes_back_to_open_when_the_session_ends() {
    let world = world().await;
    let project = world.project("alpha", Some(world.folder.path())).await;
    let task = world.task(&project, "stalled").await;
    let step = world.step(&task, "First", 0, TaskStepState::Open).await;
    let harness = harness(&world, reading(10.0));
    harness.queue.add(task.id).await.unwrap();
    harness.runner.start(&harness.queue, None).await.unwrap();
    eventually("the terminal", || async { harness.terminals.has_live_terminal_for(task.id) }).await;
    world.services.steps.transition(Some("alpha"), "stalled", Some("First"), TaskStepState::Running).await.unwrap();
    let terminal = harness.terminals.list().into_iter().next().unwrap();
    harness.terminals.write(terminal.id, b"exit\n").unwrap();

    eventually("the item to fail", || async { item_states(&harness.queue).await[0].0 == RunQueueItemState::Failed }).await;

    assert_eq!(world.step_state(&task, &step).await, TaskStepState::Open);
}

#[tokio::test]
async fn a_claim_that_ends_a_group_of_steps_renews_the_session_on_the_same_item() {
    let world = world().await;
    let project = world.project("alpha", Some(world.folder.path())).await;
    let task = world.task(&project, "grouped").await;
    world.step(&task, "Backend", 0, TaskStepState::Open).await;
    let frontend = world.step(&task, "Frontend", 1, TaskStepState::Open).await;
    world.services.step_bindings.begin_session(frontend.id, true).await.unwrap();
    let harness = harness(&world, reading(10.0));
    harness.queue.add(task.id).await.unwrap();
    harness.runner.start(&harness.queue, None).await.unwrap();
    eventually("the first session", || async { harness.terminals.has_live_terminal_for(task.id) }).await;
    let first_session = harness.terminals.list().into_iter().next().unwrap().id;

    claim(&world, "grouped", "Backend").await;

    eventually("a new session on the same task", || async {
        harness.terminals.list().into_iter().any(|terminal| terminal.task_id == Some(task.id) && terminal.id != first_session)
    })
    .await;
    assert_eq!(item_states(&harness.queue).await[0].0, RunQueueItemState::Running);
}

#[tokio::test]
async fn a_claim_by_the_session_reaches_the_console_as_a_new_queue_frame_with_the_steps_updated() {
    let world = world().await;
    let project = world.project("alpha", Some(world.folder.path())).await;
    let task = world.task(&project, "watched").await;
    world.step(&task, "One", 0, TaskStepState::Open).await;
    world.step(&task, "Two", 1, TaskStepState::Open).await;
    let harness = harness(&world, reading(10.0));
    harness.queue.add(task.id).await.unwrap();
    harness.runner.start(&harness.queue, None).await.unwrap();
    eventually("the terminal", || async { harness.terminals.has_live_terminal_for(task.id) }).await;
    let mut frames = world.events.subscribe();

    claim(&world, "watched", "One").await;

    let seen = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let event = frames.recv().await.unwrap();
            if let Some(RunQueueFrame(view)) = event.downcast::<RunQueueFrame>() {
                let steps = &view.items[0].steps;
                if steps[0].state == StepProgressState::Claimed && steps[1].current {
                    return;
                }
            }
        }
    })
    .await;
    assert!(seen.is_ok(), "a frame showing One claimed and Two current");
}
