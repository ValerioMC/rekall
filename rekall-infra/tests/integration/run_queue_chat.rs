//! The run queue on chat sessions, against a host of that kind that keeps no process: the queue
//! opens the kind its settings name, carries a task across the sessions its steps ask for, fails an
//! item whose session ends unclaimed and moves on, and says on the item when its session waits for
//! a person to answer a permission prompt.

mod support;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use rekall_core::claude::{ClaudeUsageView, InMemoryTerminalHost, Limit, SessionKind, Severity, TerminalHost, UsageReader};
use rekall_core::queue::{QueueRunner, RunQueueService};
use rekall_infra::queue::RunQueueRunner;
use rekall_kernel::Instant;
use rekall_store::{task, task_step, RunQueueItemState, RunQueueState, TaskStepState};
use sea_orm::{ActiveModelTrait, IntoActiveModel};
use support::{eventually, world, World};

/// Reads 10 % of the session limit, and counts how often it was asked.
#[derive(Default)]
struct CountedUsage(AtomicUsize);

#[async_trait::async_trait]
impl UsageReader for CountedUsage {
    async fn current(&self) -> ClaudeUsageView {
        self.0.fetch_add(1, Ordering::SeqCst);
        ClaudeUsageView::ok(
            vec![Limit {
                key: "session".into(),
                label: "Current session".into(),
                percent: 10.0,
                severity: Severity::Normal,
                resets_at: Some(Instant::now().plus(chrono::TimeDelta::hours(2))),
            }],
            Instant::now(),
        )
    }

    async fn refresh(&self) -> ClaudeUsageView {
        self.current().await
    }
}

struct Harness {
    queue: RunQueueService,
    runner: RunQueueRunner,
    host: Arc<InMemoryTerminalHost>,
    usage: Arc<CountedUsage>,
}

async fn harness(world: &World) -> Harness {
    let queue = RunQueueService::new(world.services.ctx.clone());
    let host = Arc::new(InMemoryTerminalHost::new());
    let usage = Arc::new(CountedUsage::default());
    let runner = RunQueueRunner::start(
        queue.clone(),
        host.clone(),
        usage.clone(),
        world.services.task_work.clone(),
        world.services.steps.clone(),
        world.worktrees().0,
        &world.events,
        rekall_core::system_clock(),
        Duration::from_millis(100),
        Duration::from_millis(50),
    );
    queue.update_settings(Some(80), true, None, None, SessionKind::Chat, 1).await.unwrap();
    Harness { queue, runner, host, usage }
}

async fn item_states(queue: &RunQueueService) -> Vec<(RunQueueItemState, Option<String>)> {
    queue.view().await.unwrap().items.into_iter().map(|item| (item.state, item.detail)).collect()
}

async fn step_opening_a_session(world: &World, task: &task::Model, title: &str, position: i32) -> task_step::Model {
    let mut step = task_step::Model::new(task.id, title.into(), position, Instant::now());
    step.state = TaskStepState::Open;
    step.starts_session = true;
    step.clone().into_active_model().insert(world.db()).await.unwrap();
    step
}

async fn claim(world: &World, task: &str, number: &str) {
    world.services.steps.transition(Some("alpha"), task, Some(number), TaskStepState::Claimed).await.unwrap();
}

#[tokio::test]
async fn the_kind_the_settings_name_is_kept_and_read_back() {
    let world = world().await;
    let harness = harness(&world).await;

    assert_eq!(harness.queue.view().await.unwrap().session_kind, SessionKind::Chat);
    assert_eq!(harness.queue.snapshot().await.unwrap().session_kind, SessionKind::Chat);

    let reset = harness.queue.update_settings(None, false, None, None, SessionKind::Terminal, 1).await.unwrap();
    assert_eq!(reset.session_kind, SessionKind::Terminal);
}

#[tokio::test]
async fn a_task_with_a_step_that_opens_a_session_runs_in_two_chat_sessions() {
    let world = world().await;
    let project = world.project("alpha", Some(world.folder.path())).await;
    let task = world.task(&project, "busy").await;
    world.step(&task, "First", 0, TaskStepState::Open).await;
    world.step(&task, "Second", 1, TaskStepState::Open).await;
    step_opening_a_session(&world, &task, "Third", 2).await;
    let harness = harness(&world).await;

    harness.queue.add(task.id).await.unwrap();
    harness.runner.start(&harness.queue, None).await.unwrap();
    eventually("the first chat session", || async { harness.host.openings().len() == 1 }).await;
    let first = harness.host.list().pop().unwrap();
    assert_eq!(harness.host.openings()[0].kind, SessionKind::Chat);

    claim(&world, "busy", "1").await;
    eventually("the session to stay while its group has a step left", || async { harness.host.renewed().is_empty() }).await;
    let reads_before_the_boundary = harness.usage.0.load(Ordering::SeqCst);
    claim(&world, "busy", "2").await;

    eventually("the second chat session", || async { harness.host.renewed() == vec![first.id] }).await;
    let second = harness.host.list().pop().unwrap();
    assert_ne!(second.id, first.id);
    assert_eq!(second.kind, SessionKind::Chat);
    assert_eq!(harness.host.list().len(), 1, "the first session was closed for the second");
    assert_eq!(harness.host.openings().len(), 1, "the second session is a renewal, not another opening");
    assert!(harness.usage.0.load(Ordering::SeqCst) > reads_before_the_boundary, "the ceiling was read at the boundary");

    claim(&world, "busy", "3").await;
    eventually("the queue to finish", || async { harness.queue.view().await.unwrap().state == RunQueueState::Idle }).await;
    assert_eq!(item_states(&harness.queue).await[0].0, RunQueueItemState::Finished);
    assert!(harness.host.list().is_empty());
}

#[tokio::test]
async fn a_chat_session_that_ends_before_its_work_is_claimed_fails_the_item_and_the_queue_moves_on() {
    let world = world().await;
    let project = world.project("alpha", Some(world.folder.path())).await;
    let first = world.task(&project, "first").await;
    world.step(&first, "Only", 0, TaskStepState::Open).await;
    let second = world.task(&project, "second").await;
    world.step(&second, "Only", 0, TaskStepState::Open).await;
    let harness = harness(&world).await;

    harness.queue.add(first.id).await.unwrap();
    harness.queue.add(second.id).await.unwrap();
    harness.runner.start(&harness.queue, None).await.unwrap();
    eventually("the first session", || async { harness.host.has_live_for(first.id) }).await;
    harness.host.end(harness.host.list()[0].id);

    eventually("the first item to fail and the second to run", || async {
        let states = item_states(&harness.queue).await;
        states[0].0 == RunQueueItemState::Failed && states[1].0 == RunQueueItemState::Running
    })
    .await;
    assert_eq!(item_states(&harness.queue).await[0].1.as_deref(), Some("The session ended before its work was claimed."));
    assert!(harness.host.has_live_for(second.id));
    assert_eq!(harness.host.openings().iter().map(|opening| opening.kind).collect::<Vec<_>>(), vec![SessionKind::Chat, SessionKind::Chat]);
}

#[tokio::test]
async fn a_chat_session_waiting_on_a_permission_prompt_is_a_running_item_that_says_so() {
    let world = world().await;
    let project = world.project("alpha", Some(world.folder.path())).await;
    let task = world.task(&project, "busy").await;
    world.step(&task, "Only", 0, TaskStepState::Open).await;
    let harness = harness(&world).await;

    harness.queue.add(task.id).await.unwrap();
    harness.runner.start(&harness.queue, None).await.unwrap();
    eventually("the session", || async { harness.host.has_live_for(task.id) }).await;
    let session = harness.host.list()[0].id;

    harness.host.awaiting_permission(session, true);
    eventually("the item to say it waits", || async {
        item_states(&harness.queue).await[0].1.as_deref() == Some("Waiting for you to answer a permission prompt in the chat session.")
    })
    .await;
    assert_eq!(item_states(&harness.queue).await[0].0, RunQueueItemState::Running);
    assert!(harness.host.has_live_for(task.id));

    harness.host.awaiting_permission(session, false);
    eventually("the item to stop saying it waits", || async { item_states(&harness.queue).await[0].1.is_none() }).await;
    assert_eq!(item_states(&harness.queue).await[0].0, RunQueueItemState::Running);
}
