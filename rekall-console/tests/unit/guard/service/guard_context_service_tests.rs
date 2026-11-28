use std::sync::Arc;

use rekall_core::guard::GuardLateChanges;
use rekall_core::worktree::{ScriptedWorktreeHost, WorktreeLayout};
use rekall_core::EventBus;
use rekall_kernel::{Id, Instant};
use rekall_store::{company, task_step, ProjectStatus};
use sea_orm::{ActiveModelTrait, IntoActiveModel};

use super::*;

struct World {
    service: GuardContextService,
    host: Arc<ScriptedWorktreeHost>,
    db: sea_orm::DatabaseConnection,
    task: task::Model,
    worktree: String,
    _root: tempfile::TempDir,
}

/// A task in its own worktree, which holds one uncommitted file.
async fn world(auto_commit: bool) -> World {
    let database = rekall_store::open_in_memory().await.unwrap();
    let now = Instant::now();
    let company = company::Model { id: Id::random(), name: "Acme".into(), description: None, created_at: now, updated_at: now };
    company.clone().into_active_model().insert(&database.conn).await.unwrap();
    let project = project::Model { auto_commit, company_id: company.id, ..project("vega", Some("/work/vega")) };
    project.clone().into_active_model().insert(&database.conn).await.unwrap();
    let task = task::Model::new("report".into(), "Report".into(), project.id, now);
    task.clone().into_active_model().insert(&database.conn).await.unwrap();
    let root = tempfile::tempdir().unwrap();
    let host = Arc::new(ScriptedWorktreeHost::new());
    let ctx = Ctx::new(database.conn.clone(), EventBus::new());
    let worktrees = TaskWorktreeService::new(ctx.clone(), host.clone(), WorktreeLayout::new(root.path().join("worktrees")));
    let worktree = worktrees.ensure(task.id).await.unwrap().to_string_lossy().into_owned();
    host.leave_unsaved_files("rekall/report", &["config.json"]);
    let service = GuardContextService::new(ctx, Vec::new(), worktrees);
    World { service, host, db: database.conn.clone(), task, worktree, _root: root }
}

impl World {
    async fn step(&self, position: i32, state: TaskStepState, claimed_at: Option<&str>) {
        let mut step = task_step::Model::new(self.task.id, format!("Step {position}"), position, Instant::now());
        step.state = state;
        step.claimed_at = claimed_at.map(|text| text.parse().unwrap());
        step.into_active_model().insert(&self.db).await.unwrap();
    }

    async fn late_changes_in(&self, folder: &str, scope: Option<GuardScope>) -> Option<GuardLateChanges> {
        let command = GuardContextCommand { folder: folder.to_string(), scope };
        self.service.context_of(command).await.unwrap().late_changes
    }

    async fn late_changes(&self) -> Option<GuardLateChanges> {
        self.late_changes_in(&self.worktree, Some(GuardScope::Work(self.task.id))).await
    }
}

#[tokio::test]
async fn a_claimed_task_with_uncommitted_files_names_them_and_the_step_claimed_last() {
    let world = world(true).await;
    world.step(0, TaskStepState::Claimed, Some("2026-01-02T10:00:00Z")).await;
    world.step(1, TaskStepState::Claimed, Some("2026-01-01T10:00:00Z")).await;

    let late = world.late_changes().await.unwrap();

    assert_eq!(late, GuardLateChanges { task_anchor: world.task.anchor(), files: vec!["config.json".into()], step_to_claim: Some(1) });
}

#[tokio::test]
async fn a_session_opened_by_hand_inside_the_worktree_is_told_too() {
    let world = world(true).await;
    world.step(0, TaskStepState::Claimed, Some("2026-01-01T10:00:00Z")).await;

    let inside = format!("{}/rekall-core", world.worktree);

    assert!(world.late_changes_in(&inside, None).await.is_some());
}

#[tokio::test]
async fn a_claimed_task_with_no_steps_has_no_step_to_claim_again() {
    let world = world(true).await;
    let in_its_worktree = repo::task::require(&world.db, world.task.id).await.unwrap();
    let claimed = task::Model { review_state: TaskStepState::Claimed, ..in_its_worktree };
    claimed.into_active_model().reset_all().update(&world.db).await.unwrap();

    assert_eq!(world.late_changes().await.unwrap().step_to_claim, None);
}

#[tokio::test]
async fn work_still_open_or_running_or_already_accepted_leaves_nothing_to_report() {
    for states in [[TaskStepState::Claimed, TaskStepState::Open], [TaskStepState::Claimed, TaskStepState::Running], [TaskStepState::Done, TaskStepState::Done]] {
        let world = world(true).await;
        for (position, state) in states.into_iter().enumerate() {
            world.step(position as i32, state, None).await;
        }

        assert_eq!(world.late_changes().await, None, "{states:?}");
    }
}

#[tokio::test]
async fn a_clean_worktree_or_a_project_that_does_not_commit_on_claim_reports_nothing() {
    let clean = world(true).await;
    clean.step(0, TaskStepState::Claimed, None).await;
    clean.host.leave_unsaved_files("rekall/report", &[]);
    let without_auto_commit = world(false).await;
    without_auto_commit.step(0, TaskStepState::Claimed, None).await;

    assert_eq!(clean.late_changes().await, None);
    assert_eq!(without_auto_commit.late_changes().await, None);
}

#[tokio::test]
async fn a_session_outside_the_worktree_or_launched_without_a_step_reports_nothing() {
    let world = world(true).await;
    world.step(0, TaskStepState::Claimed, None).await;

    assert_eq!(world.late_changes_in("/work/vega", Some(GuardScope::Work(world.task.id))).await, None);
    assert_eq!(world.late_changes_in("/work/vega", None).await, None);
    assert_eq!(world.late_changes_in(&world.worktree, Some(GuardScope::WithoutStep)).await, None);
}

fn project(label: &str, repo_folder: Option<&str>) -> project::Model {
    let now = Instant::now();
    project::Model {
        id: Id::random(),
        label: label.into(),
        title: label.into(),
        status: ProjectStatus::Active,
        icon: String::new(),
        description: None,
        blueprint_markdown: None,
        repo_folder: repo_folder.map(String::from),
        auto_commit: false, main_branch: None, pull_request_flow: false, task_key: String::new(),
        company_id: Id::random(),
        created_at: now,
        updated_at: now,
    }
}

fn label_holding(projects: &[project::Model], folder: &str) -> Option<String> {
    project_holding(projects, &[], folder).map(|(project, _)| project.label.clone())
}

#[test]
fn a_folder_inside_a_projects_folder_belongs_to_that_project() {
    let projects = [project("vega", Some("/work/vega")), project("orion", Some("/work/orion"))];

    assert_eq!(label_holding(&projects, "/work/vega/rekall-core/src"), Some("vega".into()));
    assert_eq!(label_holding(&projects, "/work/orion"), Some("orion".into()));
}

#[test]
fn the_innermost_project_folder_wins() {
    let projects = [project("mono", Some("/work/mono")), project("mono-ui", Some("/work/mono/ui"))];

    assert_eq!(label_holding(&projects, "/work/mono/ui/src"), Some("mono-ui".into()));
    assert_eq!(label_holding(&projects, "/work/mono/api"), Some("mono".into()));
}

#[test]
fn a_folder_that_only_shares_a_prefix_is_not_inside() {
    let projects = [project("vega", Some("/work/vega"))];

    assert_eq!(label_holding(&projects, "/work/vega-old/src"), None);
}

#[test]
fn a_project_without_a_folder_holds_nothing() {
    let projects = [project("vega", None), project("blank", Some("  "))];

    assert_eq!(label_holding(&projects, "/work/vega"), None);
}

#[test]
fn a_folder_inside_a_tasks_worktree_belongs_to_the_tasks_project() {
    let projects = [project("vega", Some("/work/vega"))];
    let mut task = task::Model::new("report".into(), "Report".into(), projects[0].id, Instant::now());
    task.worktree_folder = Some("/data/worktrees/vega/report".into());

    let worktrees = [task];
    let (owner, task) = project_holding(&projects, &worktrees, "/data/worktrees/vega/report/rekall-core").unwrap();

    assert_eq!(owner.label, "vega");
    assert_eq!(task.map(|task| task.id), Some(worktrees[0].id));
}
