use rekall_common::Instant;
use rekall_model::{company, project, task, ProjectStatus, TaskKind};
use rekall_service::EventBus;
use sea_orm::{ActiveModelTrait, DatabaseConnection, IntoActiveModel};

use super::*;
use crate::catalog::EpicActivityKind;

async fn database() -> (DatabaseConnection, Id) {
    let database = rekall_repository::open_in_memory().await.unwrap();
    let now = Instant::now();
    let company = company::Model { id: Id::random(), name: "Acme".into(), description: None, created_at: now, updated_at: now };
    company.clone().into_active_model().insert(&database.conn).await.unwrap();
    let project = project::Model {
        id: Id::random(),
        label: "app".into(),
        title: "App".into(),
        status: ProjectStatus::Active,
        icon: "folder".into(),
        description: None,
        blueprint_markdown: None,
        repo_folder: None,
        auto_commit: false,
        company_id: company.id,
        created_at: now,
        updated_at: now,
    };
    project.clone().into_active_model().insert(&database.conn).await.unwrap();
    (database.conn, project.id)
}

async fn insert_task(db: &DatabaseConnection, task: task::Model) -> task::Model {
    task.clone().into_active_model().insert(db).await.unwrap();
    task
}

#[tokio::test]
async fn the_activity_covers_the_tasks_of_the_epic_only() {
    let (db, project_id) = database().await;
    let now = Instant::now();
    let epic = insert_task(&db, task::Model { kind: TaskKind::Epic, ..task::Model::new("checkout".into(), "Checkout".into(), project_id, now) }).await;
    let mut member = task::Model::new("pay".into(), "Pay".into(), project_id, now);
    member.epic_id = Some(epic.id);
    member.claimed_at = Some(now);
    let member = insert_task(&db, member).await;
    let mut outsider = task::Model::new("login".into(), "Login".into(), project_id, now);
    outsider.claimed_at = Some(now);
    insert_task(&db, outsider).await;
    let service = EpicService::new(Ctx::new(db, EventBus::new()));

    let activity = service.activity(epic.id).await.unwrap();

    assert_eq!(activity.len(), 1);
    assert_eq!(activity[0].task_id, member.id);
    assert_eq!(activity[0].kind, EpicActivityKind::TaskClaimed);
}

#[tokio::test]
async fn a_plain_task_has_no_epic_activity() {
    let (db, project_id) = database().await;
    let plain = insert_task(&db, task::Model::new("pay".into(), "Pay".into(), project_id, Instant::now())).await;
    let service = EpicService::new(Ctx::new(db, EventBus::new()));
    assert!(service.activity(plain.id).await.is_err());
}
