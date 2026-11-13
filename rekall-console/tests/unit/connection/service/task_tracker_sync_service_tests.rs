use std::sync::Arc;

use rekall_core::connection::{InMemorySecretStore, ScriptedTaskTracker, Secret, SecretStore, TrackerFailure};
use rekall_core::{Ctx, EventBus};
use rekall_kernel::Instant;
use rekall_store::{company, external_connection, project, project_connection, task, ConnectionKind, ProjectStatus};
use sea_orm::{ActiveModelTrait, IntoActiveModel};

use super::*;

struct World {
    service: TaskTrackerSyncService,
    tracker: Arc<ScriptedTaskTracker>,
    task_id: Id,
}

async fn world() -> World {
    let database = rekall_store::open_in_memory().await.unwrap();
    let ctx = Ctx::new(database.conn.clone(), EventBus::new());
    let now = Instant::now();
    let company = company::Model { id: Id::random(), name: "Acme".into(), description: None, created_at: now, updated_at: now };
    company.clone().into_active_model().insert(&ctx.db).await.unwrap();
    let project = project::Model {
        id: Id::random(), label: "app".into(), title: "App".into(), status: ProjectStatus::Active, icon: "folder".into(), description: None,
        blueprint_markdown: None, repo_folder: None, auto_commit: false, main_branch: None, pull_request_flow: false, company_id: company.id, created_at: now, updated_at: now,
    };
    project.clone().into_active_model().insert(&ctx.db).await.unwrap();
    let mut connection = external_connection::Model::new("github".into(), "GitHub".into(), ConnectionKind::Github, Some("https://github.com".into()), now);
    connection.secret_set_at = Some(now);
    connection.clone().into_active_model().insert(&ctx.db).await.unwrap();
    let mut binding = project_connection::Model::new(project.id, connection.id, "github".into(), Some("acme/app".into()), now);
    binding.task_manager = true;
    binding.clone().into_active_model().insert(&ctx.db).await.unwrap();
    let mut task = task::Model::new("first".into(), "First".into(), project.id, now);
    task.description = Some("Do it.".into());
    task.clone().into_active_model().insert(&ctx.db).await.unwrap();
    let secrets = Arc::new(InMemorySecretStore::new());
    secrets.put(&connection.secret_ref, &Secret::new("token")).unwrap();
    let tracker = Arc::new(ScriptedTaskTracker::new());
    let services = rekall_core::Services::new(ctx.clone(), Arc::new(rekall_core::check::ScriptedCheckRunner::new()));
    let tasks = TaskService::new(ctx.clone(), services.review.clone(), services.time_entries.clone());
    let service = TaskTrackerSyncService::new(TaskTrackerService::new(ctx, secrets, tracker.clone()), tasks);
    World { service, tracker, task_id: task.id }
}

#[tokio::test]
async fn a_sync_returns_the_task_with_its_issue_and_its_content_written() {
    let world = world().await;

    let view = world.service.sync(world.task_id).await.unwrap();

    assert_eq!(view.tracker_issue_key.as_deref(), Some("acme/app-1"));
    assert!(view.tracker_content_synced);
    assert_eq!(world.tracker.created()[0].1.description, "Do it.");
}

#[tokio::test]
async fn a_refused_sync_is_an_error_and_leaves_the_content_unsynced() {
    let world = world().await;
    world.tracker.fail_with(TrackerFailure::Rejected { status: 401 });

    assert!(world.service.sync(world.task_id).await.is_err());

    world.tracker.recover();
    let view = world.service.sync(world.task_id).await.unwrap();
    assert!(view.tracker_content_synced);
}
