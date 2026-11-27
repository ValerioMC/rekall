//! The run queue's runner end to end against a stand-in `claude`: it works through the queue one
//! terminal at a time, skips what has nothing open, fails a session that ends unclaimed, holds at
//! the usage ceiling, and stops on request.

mod support;

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use rekall_core::queue::{ClaudeQueueOptions, QueueLaunch, QueueRunner, RunQueueFrame, RunQueueService};
use rekall_core::claude::{StepProgressState, TerminalMode, TerminalOpening, TerminalTarget, TerminalView};
use rekall_core::codex::{CodexClient, CodexModel, CodexSetup, CodexUsage, CodexUsageWindow};
use rekall_infra::queue::{RunQueueRunner, RunQueueUsage};
use rekall_core::claude::{ClaudeUsageView, CodexSessionOptions, Limit, SessionKind, SessionProvider, Severity, UsageReader};
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
    let usage = RunQueueUsage { claude: Arc::new(FixedUsage(Mutex::new(usage))), codex: world.codex() };
    terminal_runner(world, queue, usage, rekall_core::system_clock())
}

fn terminal_runner(world: &World, queue: RunQueueService, usage: RunQueueUsage, clock: rekall_core::Clock) -> Harness {
    let terminals = world.terminals(8);
    let runner = RunQueueRunner::start(
        queue.clone(),
        Arc::new(rekall_infra::pty::PtyTerminals::new(terminals.clone())),
        usage,
        world.services.task_work.clone(),
        world.services.steps.clone(),
        world.worktrees().0,
        &world.events,
        clock,
        Duration::from_millis(100),
        Duration::from_millis(50),
    );
    Harness { queue, runner, terminals }
}

async fn item_states(queue: &RunQueueService) -> Vec<(RunQueueItemState, Option<String>)> {
    queue.view().await.unwrap().items.into_iter().map(|item| (item.state, item.detail)).collect()
}

#[tokio::test]
async fn a_mixed_queue_dispatches_each_terminal_with_its_own_options() {
    let world = world().await;
    world.install_codex(support::STUB_CLAUDE);
    let project = world.project("alpha", Some(world.folder.path())).await;
    let claude = world.task(&project, "claude").await;
    world.step(&claude, "First", 0, TaskStepState::Open).await;
    let next_step = world.step(&claude, "Second", 1, TaskStepState::Open).await;
    world.services.step_bindings.begin_session(next_step.id, true).await.unwrap();
    let codex = world.task(&project, "codex").await;
    let mut claude_usage = reading(10.0);
    claude_usage.limits.push(Limit {
        key: "weekly_opus".into(),
        label: "Weekly · Opus".into(),
        percent: 95.0,
        severity: Severity::Normal,
        resets_at: Some(Instant::now().plus_minutes(60)),
    });
    let codex_usage = Arc::new(CodexReading { usage: Mutex::new(CodexUsage::Unavailable), refreshes: AtomicUsize::new(0) });
    codex_usage.set(10.0, Instant::now().plus_minutes(60));
    let usage = RunQueueUsage { claude: Arc::new(FixedUsage(Mutex::new(claude_usage))), codex: codex_usage };
    let queue = RunQueueService::new(world.services.ctx.clone());
    let harness = terminal_runner(&world, queue, usage, rekall_core::system_clock());
    harness.queue.update_settings(Some(80), true, Some("haiku"), Some("low"), SessionKind::Terminal, 1).await.unwrap();
    harness.queue.add(claude.id, QueueLaunch {
        claude_options: Some(ClaudeQueueOptions::new(Some("sonnet"), Some("high")).unwrap()),
        ..QueueLaunch::default()
    }).await.unwrap();
    harness.queue.add(codex.id, codex_launch()).await.unwrap();
    harness.queue.update_settings(Some(80), true, Some("opus"), Some("max"), SessionKind::Terminal, 1).await.unwrap();
    harness.runner.start(&harness.queue, None).await.unwrap();

    eventually("Claude's terminal", || async { harness.terminals.has_live_terminal_for(claude.id) }).await;
    let first = harness.terminals.list().pop().unwrap();
    assert_claude_terminal_options(&first, Some("sonnet"), Some("high"));
    claim(&world, "claude", "First").await;
    eventually("Claude's renewed terminal", || async {
        harness.terminals.list().iter().any(|session| session.live && session.task_id == Some(claude.id) && session.id != first.id)
    }).await;
    assert_claude_terminal_options(&harness.terminals.list()[0], Some("sonnet"), Some("high"));
    claim(&world, "claude", "Second").await;

    eventually("Codex's terminal", || async { harness.terminals.has_live_terminal_for(codex.id) }).await;
    let second = harness.terminals.list().pop().unwrap();
    assert_eq!(second.kind, SessionKind::Terminal);
    assert_codex_options(&second);
    harness.runner.stop().await.unwrap();
}

#[tokio::test]
async fn account_defaults_and_inherited_queue_settings_dispatch_differently() {
    let world = world().await;
    let project = world.project("alpha", Some(world.folder.path())).await;
    let delegated = world.task(&project, "delegated").await;
    world.step(&delegated, "Only", 0, TaskStepState::Open).await;
    let inherited = world.task(&project, "inherited").await;
    let harness = harness(&world, reading(10.0));
    harness.queue.add(delegated.id, QueueLaunch {
        claude_options: Some(ClaudeQueueOptions::default()),
        ..QueueLaunch::default()
    }).await.unwrap();
    harness.queue.add(inherited.id, QueueLaunch::default()).await.unwrap();
    harness.queue.update_settings(None, true, Some("opus"), Some("low"), SessionKind::Terminal, 1).await.unwrap();
    harness.runner.start(&harness.queue, None).await.unwrap();

    eventually("the task using the account defaults", || async { harness.terminals.has_live_terminal_for(delegated.id) }).await;
    assert_claude_terminal_options(&harness.terminals.list()[0], None, None);
    claim(&world, "delegated", "Only").await;

    eventually("the task inheriting queue settings", || async { harness.terminals.has_live_terminal_for(inherited.id) }).await;
    assert_claude_terminal_options(&harness.terminals.list()[0], Some("opus"), Some("low"));
    harness.runner.stop().await.unwrap();
}

fn assert_claude_terminal_options(session: &TerminalView, model: Option<&str>, effort: Option<&str>) {
    assert_eq!((session.provider, session.kind), (SessionProvider::Claude, SessionKind::Terminal));
    assert_eq!((session.model.as_deref(), session.effort.as_deref(), session.skip_permissions), (model, effort, true));
}

fn codex_launch() -> QueueLaunch {
    QueueLaunch {
        provider: SessionProvider::Codex,
        codex_options: Some(CodexSessionOptions::new(false, Some("gpt-test"), Some("high")).unwrap()),
        ..QueueLaunch::default()
    }
}

fn assert_codex_options(session: &TerminalView) {
    assert_eq!(session.provider, SessionProvider::Codex);
    assert_eq!((session.model.as_deref(), session.effort.as_deref(), session.skip_permissions), (Some("gpt-test"), Some("high"), false));
}

#[tokio::test]
async fn codex_renews_then_keeps_its_saved_options_after_stop_and_start() {
    let world = world().await;
    world.install_codex(support::STUB_CLAUDE);
    let project = world.project("alpha", Some(world.folder.path())).await;
    let task = world.task(&project, "grouped").await;
    let first = world.step(&task, "Backend", 0, TaskStepState::Open).await;
    let second = world.step(&task, "Frontend", 1, TaskStepState::Open).await;
    world.services.step_bindings.begin_session(second.id, true).await.unwrap();
    let harness = harness(&world, reading(10.0));
    harness.queue.add(task.id, codex_launch()).await.unwrap();
    harness.runner.start(&harness.queue, None).await.unwrap();
    eventually("the first Codex session", || async { harness.terminals.has_live_terminal_for(task.id) }).await;
    let original = harness.terminals.list()[0].id;

    claim(&world, "grouped", "Backend").await;
    eventually("the renewed Codex session", || async {
        harness.terminals.list().iter().any(|session| session.live && session.id != original)
    }).await;
    let renewed = harness.terminals.list().pop().unwrap();
    assert_codex_options(&renewed);

    harness.runner.stop().await.unwrap();
    harness.runner.start(&harness.queue, None).await.unwrap();
    eventually("the restarted Codex session", || async { harness.terminals.has_live_terminal_for(task.id) }).await;
    let restarted = harness.terminals.list().pop().unwrap();
    assert_ne!(renewed.id, restarted.id);
    assert_codex_options(&restarted);
    assert_eq!(world.step_state(&task, &first).await, TaskStepState::Claimed);
    assert_eq!(world.step_state(&task, &second).await, TaskStepState::Open);
    harness.runner.stop().await.unwrap();
}

#[tokio::test]
async fn codex_dispatches_from_the_saved_item_after_file_database_recovery() {
    let world = world().await;
    world.install_codex(support::STUB_CLAUDE);
    let project = world.project("alpha", Some(world.folder.path())).await;
    let task = world.task(&project, "interrupted").await;
    let first = world.step(&task, "First", 0, TaskStepState::Claimed).await;
    let second = world.step(&task, "Second", 1, TaskStepState::Open).await;
    let queue = RunQueueService::new(world.services.ctx.clone());
    let waiting = queue.add(task.id, codex_launch()).await.unwrap();
    queue.start(None).await.unwrap();
    queue.mark_item(waiting.items[0].id, RunQueueItemState::Running, None).await.unwrap();

    let reopened = rekall_store::open(world.database.path.as_deref().unwrap()).await.unwrap();
    let recovered_queue = RunQueueService::new(rekall_core::Ctx::new(reopened.conn, world.events.clone()));
    let usage = RunQueueUsage { claude: Arc::new(FixedUsage(Mutex::new(reading(10.0)))), codex: world.codex() };
    let recovered = terminal_runner(&world, recovered_queue, usage, rekall_core::system_clock());

    eventually("the recovered Codex session", || async { recovered.terminals.has_live_terminal_for(task.id) }).await;
    assert_codex_options(&recovered.terminals.list()[0]);
    assert_eq!(world.step_state(&task, &first).await, TaskStepState::Claimed);
    assert_eq!(world.step_state(&task, &second).await, TaskStepState::Open);
    recovered.runner.stop().await.unwrap();
}

#[tokio::test]
async fn a_missing_codex_cli_fails_its_item_even_with_a_usage_ceiling() {
    let world = world().await;
    let project = world.project("alpha", Some(world.folder.path())).await;
    let missing = world.task(&project, "codex").await;
    let next = world.task(&project, "claude").await;
    let harness = harness(&world, reading(10.0));
    harness.queue.update_settings(Some(80), false, None, None, SessionKind::Terminal, 1).await.unwrap();
    harness.queue.add(missing.id, codex_launch()).await.unwrap();
    harness.queue.add(next.id, QueueLaunch::default()).await.unwrap();
    harness.runner.start(&harness.queue, None).await.unwrap();

    eventually("Claude after the unavailable Codex item", || async { harness.terminals.has_live_terminal_for(next.id) }).await;
    let (state, detail) = &item_states(&harness.queue).await[0];
    assert_eq!(*state, RunQueueItemState::Failed);
    assert!(detail.as_deref().unwrap().contains("Codex CLI was not found"));
    harness.runner.stop().await.unwrap();
}

#[tokio::test]
async fn a_queued_provider_leaves_an_existing_session_on_the_task_alone() {
    let world = world().await;
    world.install_codex(support::STUB_CLAUDE);
    let project = world.project("alpha", Some(world.folder.path())).await;
    let task = world.task(&project, "one").await;
    let harness = harness(&world, reading(10.0));
    let existing = harness.terminals.open(TerminalOpening {
        target: TerminalTarget::Task(task.id),
        step_id: None,
        provider: SessionProvider::Claude,
        skip_permissions: false,
        model: None,
        effort: None,
        mode: TerminalMode::Work,
        kind: SessionKind::Terminal,
    }).await.unwrap();
    harness.queue.add(task.id, codex_launch()).await.unwrap();
    harness.runner.start(&harness.queue, None).await.unwrap();

    eventually("the queue item to fail on the conflict", || async {
        item_states(&harness.queue).await[0].0 == RunQueueItemState::Failed
    }).await;
    assert!(item_states(&harness.queue).await[0].1.as_deref().unwrap().contains("Claude Code Terminal is already open"));
    assert_eq!(harness.terminals.list()[0].id, existing.id);
    harness.terminals.close(existing.id, "Test finished").await.unwrap();
}

struct CodexReading {
    usage: Mutex<CodexUsage>,
    refreshes: AtomicUsize,
}

impl CodexReading {
    fn set(&self, used_percent: f64, resets_at: Instant) {
        *self.usage.lock().unwrap() = CodexUsage::Ok {
            primary: Some(CodexUsageWindow {
                used_percent,
                resets_at: Some(resets_at.epoch_millis() / 1000),
                window_duration_mins: Some(300),
            }),
            secondary: None,
        };
    }
}

#[async_trait::async_trait]
impl CodexClient for CodexReading {
    async fn usage(&self, refresh: bool) -> CodexUsage {
        if refresh {
            self.refreshes.fetch_add(1, Ordering::Relaxed);
        }
        self.usage.lock().unwrap().clone()
    }

    async fn models(&self) -> rekall_kernel::Result<Vec<CodexModel>> {
        unreachable!()
    }

    async fn setup(&self) -> rekall_kernel::Result<CodexSetup> {
        unreachable!()
    }

    async fn install(&self) -> rekall_kernel::Result<CodexSetup> {
        unreachable!()
    }
}

#[tokio::test]
async fn codex_usage_controls_dispatch_and_the_hold_at_a_claim_boundary() {
    let world = world().await;
    world.install_codex(support::STUB_CLAUDE);
    let project = world.project("alpha", Some(world.folder.path())).await;
    let task = world.task(&project, "grouped").await;
    let first = world.step(&task, "First", 0, TaskStepState::Open).await;
    let next = world.step(&task, "Next", 1, TaskStepState::Open).await;
    world.services.step_bindings.begin_session(next.id, true).await.unwrap();
    let queue = RunQueueService::new(world.services.ctx.clone());
    let clock = Arc::new(Mutex::new(Instant::now()));
    let read_clock = clock.clone();
    let codex = Arc::new(CodexReading { usage: Mutex::new(CodexUsage::Unavailable), refreshes: AtomicUsize::new(0) });
    let usage = RunQueueUsage { claude: Arc::new(FixedUsage(Mutex::new(reading(92.0)))), codex: codex.clone() };
    let harness = terminal_runner(&world, queue, usage, Arc::new(move || *read_clock.lock().unwrap()));
    harness.queue.update_settings(Some(80), false, None, None, SessionKind::Terminal, 1).await.unwrap();
    harness.queue.add(task.id, codex_launch()).await.unwrap();
    harness.runner.start(&harness.queue, None).await.unwrap();

    let hold_until = expect_codex_hold(&harness, "Codex usage can't be read").await;
    assert_eq!(harness.terminals.live_count(), 0);
    codex.set(15.0, hold_until.plus_seconds(120));
    *clock.lock().unwrap() = hold_until;
    eventually("Codex running despite Claude's higher usage", || async { harness.terminals.has_live_terminal_for(task.id) }).await;
    let original = harness.terminals.list()[0].id;

    codex.set(95.0, hold_until.plus_seconds(120));
    claim(&world, "grouped", "First").await;
    let resume_at = expect_codex_hold(&harness, "Codex primary window is at 95%").await;
    assert_eq!(harness.terminals.live_count(), 0);
    codex.set(10.0, resume_at.plus_seconds(600));
    *clock.lock().unwrap() = resume_at;
    eventually("Codex resuming at the next step", || async { harness.terminals.has_live_terminal_for(task.id) }).await;
    let resumed = harness.terminals.list().pop().unwrap();
    assert_ne!(resumed.id, original);
    assert_codex_options(&resumed);
    assert_eq!(codex.refreshes.load(Ordering::Relaxed), 2);
    assert_eq!(world.step_state(&task, &first).await, TaskStepState::Claimed);
    assert_eq!(world.step_state(&task, &next).await, TaskStepState::Open);
    harness.runner.stop().await.unwrap();
}

async fn expect_codex_hold(harness: &Harness, reason: &str) -> Instant {
    eventually("the queue held for Codex", || async { harness.queue.view().await.unwrap().state == RunQueueState::Holding }).await;
    let held = harness.queue.view().await.unwrap();
    assert!(held.hold_reason.as_deref().unwrap().starts_with(reason));
    assert_eq!(held.items[0].state, RunQueueItemState::Queued);
    held.hold_until.unwrap()
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

    harness.queue.add(busy.id, QueueLaunch::default()).await.unwrap();
    harness.queue.add(accepted.id, QueueLaunch::default()).await.unwrap();
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

    harness.queue.add(task.id, QueueLaunch::default()).await.unwrap();
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
    harness.queue.add(task.id, QueueLaunch::default()).await.unwrap();
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

    harness.queue.add(task.id, QueueLaunch::default()).await.unwrap();
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
    harness.queue.add(first.id, QueueLaunch::default()).await.unwrap();
    harness.queue.add(second.id, QueueLaunch::default()).await.unwrap();
    harness.runner.start(&harness.queue, None).await.unwrap();
    eventually("the first item running", || async { item_states(&harness.queue).await[0].0 == RunQueueItemState::Running }).await;

    claim(&world, "first", "One").await;
    claim(&world, "first", "Two").await;
    assert_eq!(item_states(&harness.queue).await[0].0, RunQueueItemState::Running, "open work keeps it running");
    claim(&world, "first", "Three").await;

    eventually("the second item running", || async { item_states(&harness.queue).await[1].0 == RunQueueItemState::Running }).await;
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
    harness.queue.add(flow.id, QueueLaunch::default()).await.unwrap();
    harness.queue.add(after.id, QueueLaunch::default()).await.unwrap();
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
    harness.queue.add(task.id, QueueLaunch::default()).await.unwrap();
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
    harness.queue.add(task.id, QueueLaunch::default()).await.unwrap();
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
    harness.queue.add(task.id, QueueLaunch::default()).await.unwrap();
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
    harness.queue.add(task.id, QueueLaunch::default()).await.unwrap();
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
