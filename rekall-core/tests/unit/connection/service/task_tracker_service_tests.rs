use std::sync::Arc;

use rekall_kernel::{Id, Instant};
use rekall_store::{company, external_connection, project, project_connection, task, ConnectionKind, ProjectStatus, TaskKind};
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};

use super::*;
use crate::connection::{InMemorySecretStore, ScriptedTaskTracker, TrackerFailure};
use crate::EventBus;

struct World {
    service: TaskTrackerService,
    tracker: Arc<ScriptedTaskTracker>,
    ctx: Ctx,
    project_id: Id,
}

async fn world(kind: ConnectionKind, target: Option<&str>, task_manager: bool) -> World {
    let database = rekall_store::open_in_memory().await.unwrap();
    let ctx = Ctx::new(database.conn.clone(), EventBus::new());
    let now = Instant::now();
    let company = company::Model { id: Id::random(), name: "Acme".into(), description: None, created_at: now, updated_at: now };
    company.clone().into_active_model().insert(&ctx.db).await.unwrap();
    let project = project::Model {
        id: Id::random(), label: "app".into(), title: "App".into(), status: ProjectStatus::Active, icon: "folder".into(), description: None,
        blueprint_markdown: None, repo_folder: None, auto_commit: false, main_branch: None, company_id: company.id, created_at: now, updated_at: now,
    };
    project.clone().into_active_model().insert(&ctx.db).await.unwrap();
    let mut connection = external_connection::Model::new("tracker".into(), "Tracker".into(), kind, Some("https://tracker.example.com".into()), now);
    connection.secret_set_at = Some(now);
    connection.clone().into_active_model().insert(&ctx.db).await.unwrap();
    let mut binding = project_connection::Model::new(project.id, connection.id, "tracker".into(), target.map(str::to_string), now);
    binding.task_manager = task_manager;
    binding.clone().into_active_model().insert(&ctx.db).await.unwrap();
    let secrets = Arc::new(InMemorySecretStore::new());
    secrets.put(&connection.secret_ref, &Secret::new("token")).unwrap();
    let tracker = Arc::new(ScriptedTaskTracker::new());
    World { service: TaskTrackerService::new(ctx.clone(), secrets, tracker.clone()), tracker, ctx, project_id: project.id }
}

impl World {
    async fn task(&self, label: &str, edit: impl FnOnce(&mut task::Model)) -> Id {
        let mut task = task::Model::new(label.into(), format!("Task {label}"), self.project_id, Instant::now());
        task.description = Some("Do it.".into());
        edit(&mut task);
        task.clone().into_active_model().insert(&self.ctx.db).await.unwrap();
        task.id
    }

    async fn stored(&self, id: Id) -> task::Model {
        task::Entity::find_by_id(id).one(&self.ctx.db).await.unwrap().unwrap()
    }
}

#[tokio::test]
async fn a_task_without_an_issue_creates_one_and_keeps_its_link() {
    let world = world(ConnectionKind::Gitlab, Some("acme/app"), true).await;
    let id = world.task("first", |_| {}).await;

    world.service.push(id).await.unwrap();

    let created = world.tracker.created();
    assert_eq!(created.len(), 1);
    assert_eq!(created[0].0, "acme/app");
    assert_eq!(created[0].1.title, "Task first");
    assert_eq!(created[0].1.description, "Do it.");
    let stored = world.stored(id).await;
    assert_eq!(stored.tracker_issue_key.as_deref(), Some("acme/app-1"));
    assert_eq!(stored.tracker_issue_url.as_deref(), Some("https://tracker.example.com/issues/acme/app-1"));
    assert_eq!(stored.tracker_sync_error, None);
}

#[tokio::test]
async fn a_task_that_has_an_issue_rewrites_it_instead_of_creating_another() {
    let world = world(ConnectionKind::Github, Some("acme/app"), true).await;
    let id = world.task("first", |task| task.tracker_issue_key = Some("12".into())).await;

    world.service.push(id).await.unwrap();

    assert!(world.tracker.created().is_empty());
    let updated = world.tracker.updated();
    assert_eq!(updated.len(), 1);
    assert_eq!(updated[0].0, "12");
}

#[tokio::test]
async fn a_jira_project_key_is_written_upper_case() {
    let world = world(ConnectionKind::Jira, Some("eng"), true).await;
    let id = world.task("first", |_| {}).await;

    world.service.push(id).await.unwrap();

    assert_eq!(world.tracker.created()[0].0, "ENG");
}

#[tokio::test]
async fn a_project_without_a_task_manager_writes_nothing() {
    let world = world(ConnectionKind::Gitlab, Some("acme/app"), false).await;
    let id = world.task("first", |_| {}).await;

    world.service.push(id).await.unwrap();

    assert!(world.tracker.created().is_empty());
    assert_eq!(world.stored(id).await.tracker_issue_key, None);
}

#[tokio::test]
async fn an_epic_and_a_quick_task_are_not_pushed() {
    let world = world(ConnectionKind::Gitlab, Some("acme/app"), true).await;
    let epic = world.task("epic", |task| task.kind = TaskKind::Epic).await;
    let quick = world.task("quick", |task| task.quick = true).await;

    world.service.push(epic).await.unwrap();
    world.service.push(quick).await.unwrap();

    assert!(world.tracker.created().is_empty());
}

#[tokio::test]
async fn a_refusal_is_kept_on_the_task_and_cleared_by_the_next_push_that_works() {
    let world = world(ConnectionKind::Gitlab, Some("acme/app"), true).await;
    let id = world.task("first", |_| {}).await;
    world.tracker.fail_with(TrackerFailure::Rejected { status: 401 });

    world.service.push(id).await.unwrap();

    let failed = world.stored(id).await;
    assert_eq!(failed.tracker_issue_key, None);
    assert!(failed.tracker_sync_error.unwrap().contains("rejected the stored credential (HTTP 401)"));

    world.tracker.recover();
    world.service.push(id).await.unwrap();

    let recovered = world.stored(id).await;
    assert!(recovered.tracker_issue_key.is_some());
    assert_eq!(recovered.tracker_sync_error, None);
}

#[tokio::test]
async fn a_binding_without_a_target_is_kept_as_a_failure() {
    let world = world(ConnectionKind::Gitlab, None, true).await;
    let id = world.task("first", |_| {}).await;

    world.service.push(id).await.unwrap();

    assert!(world.tracker.created().is_empty());
    assert!(world.stored(id).await.tracker_sync_error.unwrap().contains("target"));
}
