use rekall_kernel::Instant;
use rekall_store::{commit_reference, company, external_connection, project, project_connection, task_step, task_tracker_link, ConnectionKind, ProjectStatus, TaskStepState};
use sea_orm::{ActiveModelTrait, IntoActiveModel};

use super::*;
use crate::connection::{InMemorySecretStore, ScriptedPullRequestHost, ScriptedTaskTracker, Secret, SecretStore};
use crate::worktree::ScriptedWorktreeHost;
use crate::EventBus;

struct World {
    service: TaskPullRequestService,
    git: Arc<ScriptedWorktreeHost>,
    host: Arc<ScriptedPullRequestHost>,
    ctx: Ctx,
    project_id: Id,
}

async fn world(pull_request_flow: bool) -> World {
    let database = rekall_store::open_in_memory().await.unwrap();
    let ctx = Ctx::new(database.conn.clone(), EventBus::new());
    let now = Instant::now();
    let company = company::Model { id: Id::random(), name: "Acme".into(), description: None, created_at: now, updated_at: now };
    company.clone().into_active_model().insert(&ctx.db).await.unwrap();
    let project = project::Model {
        id: Id::random(), label: "app".into(), title: "App".into(), status: ProjectStatus::Active, icon: "folder".into(), description: None,
        blueprint_markdown: None, repo_folder: Some("/work/app".into()), auto_commit: true, main_branch: Some("main".into()), pull_request_flow,
        company_id: company.id, created_at: now, updated_at: now,
    };
    project.clone().into_active_model().insert(&ctx.db).await.unwrap();
    let mut connection = external_connection::Model::new("github".into(), "GitHub".into(), ConnectionKind::Github, Some("https://github.com".into()), now);
    connection.secret_set_at = Some(now);
    connection.clone().into_active_model().insert(&ctx.db).await.unwrap();
    let mut binding = project_connection::Model::new(project.id, connection.id, "github".into(), Some("acme/app".into()), now);
    binding.task_manager = true;
    binding.clone().into_active_model().insert(&ctx.db).await.unwrap();
    let secrets = Arc::new(InMemorySecretStore::new());
    secrets.put(&connection.secret_ref, &Secret::new("token")).unwrap();
    let tracker = TaskTrackerService::new(ctx.clone(), secrets, Arc::new(ScriptedTaskTracker::new()));
    let git = Arc::new(ScriptedWorktreeHost::new());
    let host = Arc::new(ScriptedPullRequestHost::new());
    let layout = WorktreeLayout::new(std::path::PathBuf::from("/worktrees"));
    let service = TaskPullRequestService::new(ctx.clone(), tracker, git.clone(), host.clone(), layout);
    World { service, git, host, ctx, project_id: project.id }
}

impl World {
    async fn claimed_task(&self, label: &str) -> task::Model {
        let mut task = task::Model::new(label.into(), format!("Task {label}"), self.project_id, Instant::now());
        task.description = Some("Do it.".into());
        task.mark_review_state(TaskStepState::Claimed);
        task.clone().into_active_model().insert(&self.ctx.db).await.unwrap();
        task
    }

    async fn stored(&self, task: &task::Model) -> Option<task_pull_request::Model> {
        repo::task_pull_request::find_of_task(&self.ctx.db, task.id).await.unwrap()
    }
}

#[tokio::test]
async fn the_first_claim_pushes_the_branch_and_opens_one_pull_request_that_the_task_keeps() {
    let world = world(true).await;
    let task = world.claimed_task("report").await;

    world.service.open_if_claimed(task.id).await.unwrap();

    assert_eq!(world.git.pushed(), ["rekall/report"]);
    let opened = world.host.opened();
    assert_eq!(opened.len(), 1);
    assert_eq!((opened[0].head.as_str(), opened[0].base.as_str(), opened[0].title.as_str(), opened[0].body.as_str()), ("rekall/report", "main", "Task report", "Do it."));
    let stored = world.stored(&task).await.unwrap();
    assert_eq!((stored.number, stored.url.as_deref(), stored.failure), (Some(1), Some("https://github.com/pull/1"), None));
}

#[tokio::test]
async fn a_second_claim_opens_no_other_pull_request_and_only_pushes_the_branch_again() {
    let world = world(true).await;
    let task = world.claimed_task("report").await;
    world.service.open_if_claimed(task.id).await.unwrap();

    world.service.open_if_claimed(task.id).await.unwrap();

    assert_eq!(world.host.opened().len(), 1);
    assert_eq!(world.stored(&task).await.unwrap().number, Some(1));
}

#[tokio::test]
async fn a_refused_push_stores_the_reason_and_opens_nothing() {
    let world = world(true).await;
    let task = world.claimed_task("report").await;
    world.git.refuse_push("rekall/report", "remote: Permission denied");

    world.service.open_if_claimed(task.id).await.unwrap();

    assert!(world.host.opened().is_empty());
    let stored = world.stored(&task).await.unwrap();
    assert_eq!((stored.number, stored.failure.as_deref()), (None, Some("The push of rekall/report was refused: remote: Permission denied")));
}

#[tokio::test]
async fn a_host_that_fails_is_kept_as_the_reason_and_cleared_by_the_pull_request_that_follows() {
    let world = world(true).await;
    let task = world.claimed_task("report").await;
    world.host.fail_with(TrackerFailure::Rejected { status: 401 });

    world.service.open_if_claimed(task.id).await.unwrap();
    assert!(world.stored(&task).await.unwrap().failure.unwrap().contains("HTTP 401"));
    world.host.recover();
    world.service.open_if_claimed(task.id).await.unwrap();

    let stored = world.stored(&task).await.unwrap();
    assert_eq!((stored.number, stored.failure), (Some(1), None));
}

#[tokio::test]
async fn the_body_names_the_issue_and_lists_the_commits_by_step() {
    let world = world(true).await;
    let task = world.claimed_task("report").await;
    let now = Instant::now();
    let link = task_tracker_link::Model { task_id: task.id, issue_key: Some("12".into()), issue_url: None, sync_error: None, created_at: now, updated_at: now };
    repo::task_tracker_link::save(&world.ctx.db, link).await.unwrap();
    let step = task_step::Model::new(task.id, "Add the port".into(), 0, now);
    step.clone().into_active_model().insert(&world.ctx.db).await.unwrap();
    let commit = commit_reference::Model { id: Id::random(), task_id: task.id, step_id: Some(step.id), commit_hash: "d044a3b1f2e9".into(), comment: "feat: Add the port".into(), diff: None, in_context: false, created_at: now };
    commit.into_active_model().insert(&world.ctx.db).await.unwrap();

    world.service.open_if_claimed(task.id).await.unwrap();

    assert_eq!(world.host.opened()[0].body, "Do it.\n\nCloses #12\n\n## Commits by step\n\n- **Add the port**: `d044a3b` feat: Add the port");
}

#[tokio::test]
async fn a_task_outside_the_flow_or_not_yet_claimed_opens_nothing() {
    let outside = world(false).await;
    let task = outside.claimed_task("report").await;
    let inside = world(true).await;
    let mut running = task::Model::new("running".into(), "Running".into(), inside.project_id, Instant::now());
    running.mark_review_state(TaskStepState::Running);
    running.clone().into_active_model().insert(&inside.ctx.db).await.unwrap();

    outside.service.open_if_claimed(task.id).await.unwrap();
    inside.service.open_if_claimed(running.id).await.unwrap();

    assert!(outside.git.pushed().is_empty() && outside.host.opened().is_empty());
    assert!(inside.git.pushed().is_empty() && inside.host.opened().is_empty());
    assert_eq!(outside.stored(&task).await, None);
}

impl World {
    async fn with_pull_request(&self, label: &str) -> task::Model {
        let task = self.claimed_task(label).await;
        self.service.open_if_claimed(task.id).await.unwrap();
        task
    }
}

#[tokio::test]
async fn an_accepted_task_has_its_pull_request_merged_once_and_its_issue_closed() {
    let world = world(true).await;
    let task = world.with_pull_request("report").await;
    let now = Instant::now();
    let link = task_tracker_link::Model { task_id: task.id, issue_key: Some("12".into()), issue_url: None, sync_error: None, created_at: now, updated_at: now };
    repo::task_tracker_link::save(&world.ctx.db, link).await.unwrap();

    let first = world.service.merge_accepted(task.id).await.unwrap();
    let second = world.service.merge_accepted(task.id).await.unwrap();

    assert_eq!((first, second), (PullRequestLanding::Merged, PullRequestLanding::AlreadyMerged));
    assert_eq!((world.host.merged(), world.host.closed_issues()), (vec![1], vec!["12".to_string()]));
    assert!(world.stored(&task).await.unwrap().merged_at.is_some());
}

#[tokio::test]
async fn a_merge_the_host_refuses_keeps_the_reason_on_the_task_and_merges_nothing() {
    let world = world(true).await;
    let task = world.with_pull_request("report").await;
    world.host.merge_answers_with(PullRequestMerge::Refused("Required status check is failing".into()));

    let landing = world.service.merge_accepted(task.id).await.unwrap();

    assert_eq!(landing, PullRequestLanding::Refused("The pull request was not merged: Required status check is failing".into()));
    let stored = world.stored(&task).await.unwrap();
    assert_eq!((stored.merged_at, stored.failure.as_deref()), (None, Some("The pull request was not merged: Required status check is failing")));
}

#[tokio::test]
async fn a_pull_request_the_host_finds_conflicting_is_left_for_the_merge_session() {
    let world = world(true).await;
    let task = world.with_pull_request("report").await;
    world.host.merge_answers_with(PullRequestMerge::Conflicting);

    assert_eq!(world.service.merge_accepted(task.id).await.unwrap(), PullRequestLanding::Conflicting { base: "main".into() });
    world.host.merge_answers_with(PullRequestMerge::Merged);

    assert_eq!(world.service.merge_accepted(task.id).await.unwrap(), PullRequestLanding::Merged);
}

#[tokio::test]
async fn a_host_that_cannot_be_reached_is_an_error_to_retry_and_the_pull_request_stays_unmerged() {
    let world = world(true).await;
    let task = world.with_pull_request("report").await;
    world.host.fail_with(TrackerFailure::Failed { reason: "it answered HTTP 502".into() });

    assert!(world.service.merge_accepted(task.id).await.is_err());
    world.host.recover();

    assert_eq!(world.service.merge_accepted(task.id).await.unwrap(), PullRequestLanding::Merged);
}

#[tokio::test]
async fn a_project_outside_the_flow_or_a_task_that_never_opened_a_pull_request_has_nothing_to_merge_through_the_host() {
    let outside = world(false).await;
    let task = outside.claimed_task("report").await;
    let inside = world(true).await;
    let never_claimed = task::Model::new("idle".into(), "Idle".into(), inside.project_id, Instant::now());
    never_claimed.clone().into_active_model().insert(&inside.ctx.db).await.unwrap();

    assert_eq!(outside.service.merge_accepted(task.id).await.unwrap(), PullRequestLanding::NotInFlow);
    assert_eq!(inside.service.merge_accepted(never_claimed.id).await.unwrap(), PullRequestLanding::NothingToMerge);
}

#[tokio::test]
async fn a_task_whose_pull_request_never_opened_is_refused_with_the_reason_it_failed() {
    let world = world(true).await;
    let task = world.claimed_task("report").await;
    world.git.refuse_push("rekall/report", "remote: Permission denied");
    world.service.open_if_claimed(task.id).await.unwrap();

    let landing = world.service.merge_accepted(task.id).await.unwrap();

    assert_eq!(landing, PullRequestLanding::Refused("The push of rekall/report was refused: remote: Permission denied".into()));
}
