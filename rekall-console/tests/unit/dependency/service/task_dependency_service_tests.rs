use rekall_core::{EventBus, Services};
use rekall_kernel::Instant;
use rekall_store::{company, project, ProjectStatus, TaskKind};

use super::*;
use crate::catalog::{TaskCommand, TaskService};

struct World {
    service: TaskDependencyService,
    tasks: TaskService,
    ctx: Ctx,
    company_id: Id,
    project_id: Id,
}

async fn world() -> World {
    let database = rekall_store::open_in_memory().await.unwrap();
    let ctx = Ctx::new(database.conn.clone(), EventBus::new());
    let now = Instant::now();
    let company = company::Model { id: Id::random(), name: "Acme".into(), description: None, created_at: now, updated_at: now };
    company.clone().into_active_model().insert(&database.conn).await.unwrap();
    let project = new_project(&ctx, company.id, "app").await;
    let services = Services::new(ctx.clone(), std::sync::Arc::new(rekall_core::check::ScriptedCheckRunner::new()));
    let tasks = TaskService::new(ctx.clone(), services.review.clone(), services.time_entries.clone());
    let service = TaskDependencyService::new(ctx.clone(), RunQueueService::new(ctx.clone()));
    World { service, tasks, ctx, company_id: company.id, project_id: project }
}

async fn new_project(ctx: &Ctx, company_id: Id, label: &str) -> Id {
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
    project.clone().into_active_model().insert(&ctx.db).await.unwrap();
    project.id
}

async fn task_in(world: &World, project_id: Id, label: &str, kind: TaskKind) -> Id {
    world
        .tasks
        .create(TaskCommand { label: Some(label.into()), title: Some(label.into()), project_id: Some(project_id), kind: Some(kind), ..TaskCommand::default() })
        .await
        .unwrap()
        .id
}

async fn task(world: &World, label: &str) -> Id {
    task_in(world, world.project_id, label, TaskKind::Task).await
}

fn on(task_id: Id) -> TaskDependencyCommand {
    TaskDependencyCommand { depends_on_task_id: task_id }
}

#[tokio::test]
async fn a_dependency_is_listed_with_the_task_it_waits_for_and_that_it_is_not_met() {
    let world = world().await;
    let (report, export) = (task(&world, "report").await, task(&world, "export").await);

    let views = world.service.add(export, on(report)).await.unwrap();

    assert_eq!(views.len(), 1);
    assert_eq!((views[0].task_id, views[0].anchor.as_str(), views[0].met), (report, "project:app task:report", false));
    assert_eq!(world.service.list_of(export).await.unwrap(), views);
}

#[tokio::test]
async fn a_task_cannot_depend_on_itself() {
    let world = world().await;
    let export = task(&world, "export").await;

    let refused = world.service.add(export, on(export)).await.unwrap_err();

    assert!(matches!(refused, RekallError::IllegalArgument(_)));
}

#[tokio::test]
async fn a_dependency_that_closes_a_cycle_through_several_tasks_is_refused() {
    let world = world().await;
    let (a, b, c) = (task(&world, "a").await, task(&world, "b").await, task(&world, "c").await);
    world.service.add(b, on(a)).await.unwrap();
    world.service.add(c, on(b)).await.unwrap();

    let refused = world.service.add(a, on(c)).await.unwrap_err();

    assert!(matches!(refused, RekallError::IllegalArgument(_)));
    assert!(world.service.list_of(a).await.unwrap().is_empty());
}

#[tokio::test]
async fn a_task_of_another_project_is_refused() {
    let world = world().await;
    let other_project = new_project(&world.ctx, world.company_id, "site").await;
    let (export, elsewhere) = (task(&world, "export").await, task_in(&world, other_project, "banner", TaskKind::Task).await);

    let refused = world.service.add(export, on(elsewhere)).await.unwrap_err();

    assert!(matches!(refused, RekallError::IllegalArgument(_)));
}

#[tokio::test]
async fn an_epic_takes_no_part_on_either_side() {
    let world = world().await;
    let (export, epic) = (task(&world, "export").await, task_in(&world, world.project_id, "release", TaskKind::Epic).await);

    let waiting_for_it = world.service.add(export, on(epic)).await.unwrap_err();
    let waiting_itself = world.service.add(epic, on(export)).await.unwrap_err();

    assert!(matches!(waiting_for_it, RekallError::IllegalArgument(_)));
    assert!(matches!(waiting_itself, RekallError::IllegalArgument(_)));
}

#[tokio::test]
async fn the_same_dependency_twice_is_a_conflict() {
    let world = world().await;
    let (report, export) = (task(&world, "report").await, task(&world, "export").await);
    world.service.add(export, on(report)).await.unwrap();

    let refused = world.service.add(export, on(report)).await.unwrap_err();

    assert!(matches!(refused, RekallError::Conflict(_)));
}

#[tokio::test]
async fn a_dependency_is_removed_and_a_missing_one_is_not_found() {
    let world = world().await;
    let (report, export) = (task(&world, "report").await, task(&world, "export").await);
    world.service.add(export, on(report)).await.unwrap();

    let views = world.service.remove(export, report).await.unwrap();
    let missing = world.service.remove(export, report).await.unwrap_err();

    assert!(views.is_empty());
    assert!(matches!(missing, RekallError::NotFound(_)));
}
