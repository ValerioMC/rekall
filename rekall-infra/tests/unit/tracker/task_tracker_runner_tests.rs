use std::sync::Arc;
use std::time::Duration;

use rekall_core::connection::{InMemorySecretStore, ScriptedTaskTracker, Secret, SecretStore};
use rekall_core::Ctx;
use rekall_kernel::Instant;
use rekall_store::{company, external_connection, project, project_connection, task, ConnectionKind, ProjectStatus};
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};

use super::*;

async fn a_task_of_a_project_with_a_task_manager(ctx: &Ctx) -> Id {
    let now = Instant::now();
    let company = company::Model { id: Id::random(), name: "Acme".into(), description: None, created_at: now, updated_at: now };
    company.clone().into_active_model().insert(&ctx.db).await.unwrap();
    let project = project::Model {
        id: Id::random(), label: "app".into(), title: "App".into(), status: ProjectStatus::Active, icon: "folder".into(), description: None,
        blueprint_markdown: None, repo_folder: None, auto_commit: false, main_branch: None, company_id: company.id, created_at: now, updated_at: now,
    };
    project.clone().into_active_model().insert(&ctx.db).await.unwrap();
    let mut connection = external_connection::Model::new("gitlab".into(), "GitLab".into(), ConnectionKind::Gitlab, Some("https://gitlab.example.com".into()), now);
    connection.secret_set_at = Some(now);
    connection.clone().into_active_model().insert(&ctx.db).await.unwrap();
    let mut binding = project_connection::Model::new(project.id, connection.id, "gitlab".into(), Some("acme/app".into()), now);
    binding.task_manager = true;
    binding.clone().into_active_model().insert(&ctx.db).await.unwrap();
    let task = task::Model::new("first".into(), "First".into(), project.id, now);
    task.clone().into_active_model().insert(&ctx.db).await.unwrap();
    task.id
}

#[tokio::test]
async fn a_task_whose_content_changed_is_written_to_the_task_manager() {
    let database = rekall_store::open_in_memory().await.unwrap();
    let events = EventBus::new();
    let ctx = Ctx::new(database.conn.clone(), events.clone());
    let task_id = a_task_of_a_project_with_a_task_manager(&ctx).await;
    let secrets = Arc::new(InMemorySecretStore::new());
    for account in external_connection::Entity::find().all(&ctx.db).await.unwrap().iter().map(|connection| connection.secret_ref.clone()) {
        secrets.put(&account, &Secret::new("glpat")).unwrap();
    }
    let tracker = Arc::new(ScriptedTaskTracker::new());
    TaskTrackerRunner::start(&events, TaskTrackerService::new(ctx.clone(), secrets, tracker.clone()));

    events.publish(TaskContentChangedEvent { task_id });

    for _ in 0..100 {
        if !tracker.created().is_empty() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert_eq!(tracker.created().len(), 1);
}
