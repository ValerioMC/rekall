use rekall_kernel::Instant;
use rekall_store::{company, ProjectStatus, TaskKind};
use rekall_core::{EventBus, Services};
use sea_orm::{ActiveModelTrait, IntoActiveModel};

use super::*;

struct World {
    service: TaskService,
    steps: rekall_core::step::TaskStepService,
    project_id: Id,
    other_project_id: Id,
}

async fn world() -> World {
    let database = rekall_store::open_in_memory().await.unwrap();
    let ctx = Ctx::new(database.conn.clone(), EventBus::new());
    let now = Instant::now();
    let company = company::Model { id: Id::random(), name: "Acme".into(), description: None, created_at: now, updated_at: now };
    company.clone().into_active_model().insert(&database.conn).await.unwrap();
    let project_id = insert_project(&database.conn, company.id, "app").await;
    let other_project_id = insert_project(&database.conn, company.id, "site").await;
    let services = Services::new(ctx.clone());
    let service = TaskService::new(ctx, services.review.clone(), services.time_entries.clone());
    World { service, steps: services.steps.clone(), project_id, other_project_id }
}

async fn insert_project(db: &sea_orm::DatabaseConnection, company_id: Id, label: &str) -> Id {
    let now = Instant::now();
    let project = project::Model {
        id: Id::random(),
        label: label.into(),
        title: label.into(),
        status: ProjectStatus::Active,
        icon: "folder".into(),
        description: None,
        blueprint_markdown: None,
        repo_folder: None,
        auto_commit: false,
        company_id,
        created_at: now,
        updated_at: now,
    };
    project.clone().into_active_model().insert(db).await.unwrap();
    project.id
}

fn command(label: &str, project_id: Id, kind: TaskKind, epic_id: Option<Id>) -> TaskCommand {
    TaskCommand {
        label: Some(label.into()),
        title: Some(label.into()),
        project_id: Some(project_id),
        kind: Some(kind),
        epic_id,
        ..TaskCommand::default()
    }
}

#[tokio::test]
async fn a_task_created_under_an_epic_names_it() {
    let world = world().await;
    let epic = world.service.create(command("checkout", world.project_id, TaskKind::Epic, None)).await.unwrap();
    let task = world.service.create(command("pay", world.project_id, TaskKind::Task, Some(epic.id))).await.unwrap();
    assert_eq!(epic.kind, TaskKind::Epic);
    assert_eq!(task.epic_id, Some(epic.id));
}

#[tokio::test]
async fn a_task_cannot_belong_to_a_plain_task() {
    let world = world().await;
    let plain = world.service.create(command("checkout", world.project_id, TaskKind::Task, None)).await.unwrap();
    let refused = world.service.create(command("pay", world.project_id, TaskKind::Task, Some(plain.id))).await;
    assert!(refused.is_err());
}

#[tokio::test]
async fn a_task_cannot_belong_to_an_epic_of_another_project() {
    let world = world().await;
    let epic = world.service.create(command("checkout", world.other_project_id, TaskKind::Epic, None)).await.unwrap();
    let refused = world.service.create(command("pay", world.project_id, TaskKind::Task, Some(epic.id))).await;
    assert!(refused.is_err());
}

#[tokio::test]
async fn an_epic_with_tasks_cannot_turn_back_into_a_task() {
    let world = world().await;
    let epic = world.service.create(command("checkout", world.project_id, TaskKind::Epic, None)).await.unwrap();
    world.service.create(command("pay", world.project_id, TaskKind::Task, Some(epic.id))).await.unwrap();
    let refused = world.service.update(epic.id, command("checkout", world.project_id, TaskKind::Task, None)).await;
    assert!(refused.is_err());
}

#[tokio::test]
async fn an_epic_with_tasks_cannot_move_to_another_project() {
    let world = world().await;
    let epic = world.service.create(command("checkout", world.project_id, TaskKind::Epic, None)).await.unwrap();
    world.service.create(command("pay", world.project_id, TaskKind::Task, Some(epic.id))).await.unwrap();
    let refused = world.service.update(epic.id, command("checkout", world.other_project_id, TaskKind::Epic, None)).await;
    assert!(refused.is_err());
}

#[tokio::test]
async fn an_empty_epic_turns_back_into_a_task() {
    let world = world().await;
    let epic = world.service.create(command("checkout", world.project_id, TaskKind::Epic, None)).await.unwrap();
    let task = world.service.update(epic.id, command("checkout", world.project_id, TaskKind::Task, None)).await.unwrap();
    assert_eq!(task.kind, TaskKind::Task);
}

#[tokio::test]
async fn a_task_with_steps_cannot_become_an_epic() {
    let world = world().await;
    let task = world.service.create(command("checkout", world.project_id, TaskKind::Task, None)).await.unwrap();
    world.steps.add(task.id, Some("Wire the form"), None).await.unwrap();
    let refused = world.service.update(task.id, command("checkout", world.project_id, TaskKind::Epic, None)).await;
    assert!(refused.is_err());
}

#[tokio::test]
async fn an_epic_takes_no_step() {
    let world = world().await;
    let epic = world.service.create(command("checkout", world.project_id, TaskKind::Epic, None)).await.unwrap();
    assert!(world.steps.add(epic.id, Some("Wire the form"), None).await.is_err());
}

#[tokio::test]
async fn deleting_an_epic_leaves_its_tasks_on_their_own() {
    let world = world().await;
    let epic = world.service.create(command("checkout", world.project_id, TaskKind::Epic, None)).await.unwrap();
    let task = world.service.create(command("pay", world.project_id, TaskKind::Task, Some(epic.id))).await.unwrap();
    world.service.delete(epic.id).await.unwrap();
    assert_eq!(world.service.get(task.id).await.unwrap().epic_id, None);
}
