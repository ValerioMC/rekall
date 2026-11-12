//! The run queue working several tasks at once: each in a worktree of its own, never two tasks
//! that depend on each other, every session closed on stop, and a worktree git refuses leaving
//! its task waiting with the reason while the others go on.

mod support;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use rekall_core::claude::{ClaudeUsageView, InMemoryTerminalHost, Limit, SessionKind, Severity, TerminalHost, TerminalMode, UsageReader};
use rekall_core::queue::{QueueRunner, RunQueueService};
use rekall_core::connection::{PullRequestMerge, ScriptedPullRequestHost};
use rekall_core::worktree::{ScriptedWorktreeHost, TaskPullRequestService, TaskWorktreeService};
use rekall_infra::queue::RunQueueRunner;
use rekall_infra::worktree::WorktreeCleaner;
use rekall_kernel::Instant;
use rekall_store::{task, task_dependency, RunQueueItemState, RunQueueState, TaskStepState};
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};
use support::{eventually, world, World};

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
    git: Arc<ScriptedWorktreeHost>,
}

async fn harness(world: &World, parallel: i32) -> Harness {
    let queue = RunQueueService::new(world.services.ctx.clone());
    let terminals = world.terminals(8);
    let (worktrees, git) = world.worktrees();
    let runner = RunQueueRunner::start(
        queue.clone(),
        Arc::new(rekall_infra::pty::PtyTerminals::new(terminals.clone())),
        Arc::new(FixedUsage(Mutex::new(reading(10.0)))),
        world.services.task_work.clone(),
        world.services.steps.clone(),
        worktrees,
        &world.events,
        rekall_core::system_clock(),
        Duration::from_millis(100),
        Duration::from_millis(50),
    );
    queue.update_settings(None, false, None, None, SessionKind::Terminal, parallel).await.unwrap();
    Harness { queue, runner, terminals, git }
}

fn start_cleaner(world: &World, worktrees: TaskWorktreeService) -> Arc<InMemoryTerminalHost> {
    start_cleaner_with(world, worktrees, world.pull_requests())
}

fn start_cleaner_with(world: &World, worktrees: TaskWorktreeService, pull_requests: TaskPullRequestService) -> Arc<InMemoryTerminalHost> {
    let sessions = Arc::new(InMemoryTerminalHost::new());
    WorktreeCleaner::start(&world.events, world.services.task_work.clone(), worktrees, pull_requests, sessions.clone(), Duration::from_millis(50));
    sessions
}

async fn item_states(queue: &RunQueueService) -> Vec<(RunQueueItemState, Option<String>)> {
    queue.view().await.unwrap().items.into_iter().map(|item| (item.state, item.detail)).collect()
}

async fn open_task(world: &World, project: &rekall_store::project::Model, label: &str) -> task::Model {
    let task = world.task(project, label).await;
    world.step(&task, "First", 0, TaskStepState::Open).await;
    task
}

async fn worktree_of(world: &World, task: &task::Model) -> Option<String> {
    task::Entity::find_by_id(task.id).one(world.db()).await.unwrap().unwrap().worktree_folder
}

#[tokio::test]
async fn two_independent_tasks_run_together_each_in_its_own_worktree_and_the_third_waits() {
    let world = world().await;
    let project = world.project("alpha", Some(world.folder.path())).await;
    let (one, two, three) = (open_task(&world, &project, "one").await, open_task(&world, &project, "two").await, open_task(&world, &project, "three").await);
    let harness = harness(&world, 2).await;

    for task in [&one, &two, &three] {
        harness.queue.add(task.id).await.unwrap();
    }
    harness.runner.start(&harness.queue, None).await.unwrap();

    eventually("two tasks running", || async {
        harness.terminals.has_live_terminal_for(one.id) && harness.terminals.has_live_terminal_for(two.id)
    })
    .await;
    assert!(!harness.terminals.has_live_terminal_for(three.id), "the third has no place yet");
    let states: Vec<_> = item_states(&harness.queue).await.into_iter().map(|(state, _)| state).collect();
    assert_eq!(states, vec![RunQueueItemState::Running, RunQueueItemState::Running, RunQueueItemState::Queued]);
    assert_eq!(harness.git.branches(), vec!["rekall/one".to_string(), "rekall/two".to_string()]);
    assert!(worktree_of(&world, &one).await.is_some_and(|folder| folder.ends_with("alpha/one")));
    assert_eq!(worktree_of(&world, &three).await, None);

    world.services.steps.transition(Some("alpha"), "one", Some("1"), TaskStepState::Claimed).await.unwrap();

    eventually("the third taking the free place", || async { harness.terminals.has_live_terminal_for(three.id) }).await;
    assert!(harness.terminals.has_live_terminal_for(two.id), "the other session is not disturbed");
}

#[tokio::test]
async fn a_task_that_depends_on_a_running_one_does_not_start() {
    let world = world().await;
    let project = world.project("alpha", Some(world.folder.path())).await;
    let (first, second) = (open_task(&world, &project, "first").await, open_task(&world, &project, "second").await);
    task_dependency::Model::new(second.id, first.id, Instant::now()).into_active_model().insert(world.db()).await.unwrap();
    let harness = harness(&world, 2).await;

    harness.queue.add(second.id).await.unwrap();
    harness.queue.add(first.id).await.unwrap();
    harness.runner.start(&harness.queue, None).await.unwrap();

    eventually("the first task running", || async { harness.terminals.has_live_terminal_for(first.id) }).await;
    tokio::time::sleep(Duration::from_millis(400)).await;
    assert!(!harness.terminals.has_live_terminal_for(second.id));
    assert_eq!(harness.queue.view().await.unwrap().state, RunQueueState::Running);
}

#[tokio::test]
async fn stopping_closes_every_open_session_and_puts_their_items_back() {
    let world = world().await;
    let project = world.project("alpha", Some(world.folder.path())).await;
    let (one, two) = (open_task(&world, &project, "one").await, open_task(&world, &project, "two").await);
    let harness = harness(&world, 2).await;
    harness.queue.add(one.id).await.unwrap();
    harness.queue.add(two.id).await.unwrap();
    harness.runner.start(&harness.queue, None).await.unwrap();
    eventually("both running", || async {
        harness.terminals.has_live_terminal_for(one.id) && harness.terminals.has_live_terminal_for(two.id)
    })
    .await;

    let stopped = harness.runner.stop().await.unwrap();

    assert_eq!(stopped.state, RunQueueState::Idle);
    assert!(!harness.terminals.has_live_terminal_for(one.id) && !harness.terminals.has_live_terminal_for(two.id));
    let states: Vec<_> = item_states(&harness.queue).await.into_iter().map(|(state, _)| state).collect();
    assert_eq!(states, vec![RunQueueItemState::Queued, RunQueueItemState::Queued]);
}

#[tokio::test]
async fn a_worktree_git_refuses_leaves_its_task_waiting_with_the_reason_and_the_other_goes_on() {
    let world = world().await;
    let project = world.project("alpha", Some(world.folder.path())).await;
    let (broken, fine) = (open_task(&world, &project, "broken").await, open_task(&world, &project, "fine").await);
    let harness = harness(&world, 2).await;
    harness.git.refuse("rekall/broken");
    harness.queue.add(broken.id).await.unwrap();
    harness.queue.add(fine.id).await.unwrap();

    harness.runner.start(&harness.queue, None).await.unwrap();

    eventually("the other task running", || async { harness.terminals.has_live_terminal_for(fine.id) }).await;
    let states = item_states(&harness.queue).await;
    assert_eq!(states[0].0, RunQueueItemState::Queued);
    assert!(states[0].1.as_deref().is_some_and(|reason| reason.starts_with("Could not make a worktree:")), "{states:?}");
    assert!(!harness.terminals.has_live_terminal_for(broken.id));
}

#[tokio::test]
async fn one_session_at_a_time_makes_no_worktree() {
    let world = world().await;
    let project = world.project("alpha", Some(world.folder.path())).await;
    let (one, two) = (open_task(&world, &project, "one").await, open_task(&world, &project, "two").await);
    let harness = harness(&world, 1).await;
    harness.queue.add(one.id).await.unwrap();
    harness.queue.add(two.id).await.unwrap();

    harness.runner.start(&harness.queue, None).await.unwrap();

    eventually("the first task running", || async { harness.terminals.has_live_terminal_for(one.id) }).await;
    assert!(!harness.terminals.has_live_terminal_for(two.id));
    assert!(harness.git.branches().is_empty());
    assert_eq!(worktree_of(&world, &one).await, None);
}

#[tokio::test]
async fn accepting_a_task_removes_its_clean_worktree_and_the_task_forgets_it() {
    let world = world().await;
    let (worktrees, git) = world.worktrees();
    let project = world.project("alpha", Some(world.folder.path())).await;
    let task = world.task(&project, "one").await;
    let step = world.step(&task, "First", 0, TaskStepState::Claimed).await;
    let folder = worktrees.ensure(task.id).await.unwrap();
    let _ = start_cleaner(&world, worktrees);

    world.services.steps.edit(step.id, None, None, Some(true), None).await.unwrap();

    eventually("the worktree to go", || async { worktree_of(&world, &task).await.is_none() }).await;
    assert!(!folder.exists());
    assert!(git.branches().is_empty());
}

#[tokio::test]
async fn accepting_a_task_merges_its_branch_before_its_worktree_goes() {
    let world = world().await;
    let (worktrees, git) = world.worktrees();
    let project = world.project("alpha", Some(world.folder.path())).await;
    let task = world.task(&project, "one").await;
    let step = world.step(&task, "First", 0, TaskStepState::Claimed).await;
    worktrees.ensure(task.id).await.unwrap();
    git.leave_commits("rekall/one");
    let _ = start_cleaner(&world, worktrees);

    world.services.steps.edit(step.id, None, None, Some(true), None).await.unwrap();

    eventually("the worktree to go", || async { worktree_of(&world, &task).await.is_none() }).await;
    assert_eq!(git.merged(), vec!["rekall/one".to_string()]);
}

#[tokio::test]
async fn accepting_a_task_on_the_pull_request_flow_merges_its_pull_request_and_not_its_branch() {
    let world = world().await;
    let (worktrees, git) = world.worktrees();
    let host = Arc::new(ScriptedPullRequestHost::new());
    let project = world.project("alpha", Some(world.folder.path())).await;
    world.run_pull_request_flow(&project).await;
    let task = world.task(&project, "one").await;
    let step = world.step(&task, "First", 0, TaskStepState::Claimed).await;
    worktrees.ensure(task.id).await.unwrap();
    git.leave_commits("rekall/one");
    let link = rekall_store::task_tracker_link::Model { task_id: task.id, issue_key: Some("12".into()), issue_url: None, sync_error: None, created_at: Instant::now(), updated_at: Instant::now() };
    rekall_store::repository::task_tracker_link::save(world.db(), link).await.unwrap();
    let _ = start_cleaner_with(&world, worktrees, world.pull_requests_over(git.clone(), host.clone()));

    world.services.steps.edit(step.id, None, None, Some(true), None).await.unwrap();

    eventually("the worktree to go", || async { worktree_of(&world, &task).await.is_none() }).await;
    assert_eq!((host.opened().len(), host.merged(), host.closed_issues()), (1, vec![1], vec!["12".to_string()]));
    assert_eq!((git.pushed(), git.merged()), (vec!["rekall/one".to_string()], Vec::<String>::new()));
}

#[tokio::test]
async fn a_pull_request_the_host_refuses_keeps_the_worktree_and_records_why() {
    let world = world().await;
    let (worktrees, git) = world.worktrees();
    let host = Arc::new(ScriptedPullRequestHost::new());
    host.merge_answers_with(PullRequestMerge::Refused("Required status check is failing".into()));
    let project = world.project("alpha", Some(world.folder.path())).await;
    world.run_pull_request_flow(&project).await;
    let task = world.task(&project, "one").await;
    let step = world.step(&task, "First", 0, TaskStepState::Claimed).await;
    worktrees.ensure(task.id).await.unwrap();
    let _ = start_cleaner_with(&world, worktrees, world.pull_requests_over(git, host.clone()));

    world.services.steps.edit(step.id, None, None, Some(true), None).await.unwrap();

    eventually("the reason to be kept", || async {
        let stored = rekall_store::repository::task_pull_request::find_of_task(world.db(), task.id).await.unwrap();
        stored.and_then(|stored| stored.failure).is_some_and(|failure| failure.contains("Required status check is failing"))
    })
    .await;
    assert!(worktree_of(&world, &task).await.is_some());
    assert!(host.merged().is_empty());
}

#[tokio::test]
async fn accepting_a_task_whose_branch_conflicts_opens_one_merge_session_and_keeps_the_worktree() {
    let world = world().await;
    let (worktrees, git) = world.worktrees();
    let project = world.project("alpha", Some(world.folder.path())).await;
    let task = world.task(&project, "one").await;
    let step = world.step(&task, "First", 0, TaskStepState::Claimed).await;
    let folder = worktrees.ensure(task.id).await.unwrap();
    git.leave_commits("rekall/one");
    git.conflict_on("rekall/one");
    let sessions = start_cleaner(&world, worktrees);

    world.services.steps.edit(step.id, None, None, Some(true), None).await.unwrap();

    eventually("the merge session", || async { sessions.has_live_for(task.id) }).await;
    tokio::time::sleep(Duration::from_millis(300)).await;
    let openings = sessions.openings();
    assert_eq!(openings.len(), 1);
    assert_eq!(openings[0].mode, TerminalMode::Merge);
    assert!(git.merged().is_empty());
    assert!(folder.exists());
}

#[tokio::test]
async fn a_conflicting_pull_request_opens_one_merge_session_and_merges_once_the_session_has_pushed() {
    let world = world().await;
    let (worktrees, git) = world.worktrees();
    let host = Arc::new(ScriptedPullRequestHost::new());
    host.merge_answers_with(PullRequestMerge::Conflicting);
    let project = world.project("alpha", Some(world.folder.path())).await;
    world.run_pull_request_flow(&project).await;
    let task = world.task(&project, "one").await;
    let step = world.step(&task, "First", 0, TaskStepState::Claimed).await;
    let folder = worktrees.ensure(task.id).await.unwrap();
    let sessions = start_cleaner_with(&world, worktrees, world.pull_requests_over(git, host.clone()));
    world.services.steps.edit(step.id, None, None, Some(true), None).await.unwrap();
    eventually("the merge session", || async { sessions.has_live_for(task.id) }).await;
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(sessions.openings().len(), 1);
    assert_eq!(sessions.openings()[0].mode, TerminalMode::MergePullRequest { base: "main".into() });
    assert!(folder.exists() && host.merged().is_empty());

    host.merge_answers_with(PullRequestMerge::Merged);

    eventually("the worktree to go", || async { worktree_of(&world, &task).await.is_none() }).await;
    assert_eq!(host.merged(), vec![1]);
    assert!(!sessions.has_live_for(task.id));
    assert_eq!(sessions.openings().len(), 1);
}

#[tokio::test]
async fn once_the_merge_session_resolves_the_conflict_the_branch_merges_and_the_worktree_goes() {
    let world = world().await;
    let (worktrees, git) = world.worktrees();
    let project = world.project("alpha", Some(world.folder.path())).await;
    let task = world.task(&project, "one").await;
    let step = world.step(&task, "First", 0, TaskStepState::Claimed).await;
    let folder = worktrees.ensure(task.id).await.unwrap();
    git.leave_commits("rekall/one");
    git.conflict_on("rekall/one");
    let sessions = start_cleaner(&world, worktrees);
    world.services.steps.edit(step.id, None, None, Some(true), None).await.unwrap();
    eventually("the merge session", || async { sessions.has_live_for(task.id) }).await;

    git.resolve_conflict("rekall/one");

    eventually("the worktree to go", || async { worktree_of(&world, &task).await.is_none() }).await;
    assert_eq!(git.merged(), vec!["rekall/one".to_string()]);
    assert!(!sessions.has_live_for(task.id));
    assert!(!folder.exists());
}

#[tokio::test]
async fn a_merge_session_that_ends_with_the_conflict_unresolved_is_not_opened_again() {
    let world = world().await;
    let (worktrees, git) = world.worktrees();
    let project = world.project("alpha", Some(world.folder.path())).await;
    let task = world.task(&project, "one").await;
    let step = world.step(&task, "First", 0, TaskStepState::Claimed).await;
    let folder = worktrees.ensure(task.id).await.unwrap();
    git.leave_commits("rekall/one");
    git.conflict_on("rekall/one");
    let sessions = start_cleaner(&world, worktrees);
    world.services.steps.edit(step.id, None, None, Some(true), None).await.unwrap();
    eventually("the merge session", || async { sessions.has_live_for(task.id) }).await;

    sessions.end(sessions.list()[0].id);

    tokio::time::sleep(Duration::from_millis(400)).await;
    assert_eq!(sessions.openings().len(), 1);
    assert!(folder.exists());
}

#[tokio::test]
async fn accepting_a_task_whose_worktree_has_unsaved_changes_leaves_it_alone() {
    let world = world().await;
    let (worktrees, git) = world.worktrees();
    let project = world.project("alpha", Some(world.folder.path())).await;
    let task = world.task(&project, "one").await;
    let step = world.step(&task, "First", 0, TaskStepState::Claimed).await;
    let folder = worktrees.ensure(task.id).await.unwrap();
    git.leave_unsaved_changes("rekall/one");
    let _ = start_cleaner(&world, worktrees);

    world.services.steps.edit(step.id, None, None, Some(true), None).await.unwrap();

    tokio::time::sleep(Duration::from_millis(400)).await;
    assert!(folder.exists());
    assert!(worktree_of(&world, &task).await.is_some());
}

#[tokio::test]
async fn a_task_with_work_still_open_keeps_its_worktree() {
    let world = world().await;
    let (worktrees, _) = world.worktrees();
    let project = world.project("alpha", Some(world.folder.path())).await;
    let task = world.task(&project, "one").await;
    let done = world.step(&task, "First", 0, TaskStepState::Claimed).await;
    world.step(&task, "Second", 1, TaskStepState::Open).await;
    let folder = worktrees.ensure(task.id).await.unwrap();
    let _ = start_cleaner(&world, worktrees);

    world.services.steps.edit(done.id, None, None, Some(true), None).await.unwrap();

    tokio::time::sleep(Duration::from_millis(400)).await;
    assert!(folder.exists());
}
