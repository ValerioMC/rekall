use rekall_kernel::Instant;
use rekall_store::{company, project, ProjectStatus, TaskStatus};
use rekall_core::queue::RunQueueService;
use rekall_core::step::NewStep;
use rekall_core::{EventBus, Services};
use sea_orm::{ActiveModelTrait, IntoActiveModel};

use super::*;
use crate::catalog::{TaskCommand, TaskService};
use crate::dependency::{TaskDependencyCommand, TaskDependencyService};

struct World {
    ctx: Ctx,
    tasks: TaskService,
    conversion: TaskEpicConversionService,
    dependencies: TaskDependencyService,
    steps: rekall_core::step::TaskStepService,
    project_id: Id,
}

async fn world() -> World {
    let database = rekall_store::open_in_memory().await.unwrap();
    let ctx = Ctx::new(database.conn.clone(), EventBus::new());
    let now = Instant::now();
    let company = company::Model { id: Id::random(), name: "Acme".into(), description: None, created_at: now, updated_at: now };
    company.clone().into_active_model().insert(&database.conn).await.unwrap();
    let project = project::Model {
        id: Id::random(),
        label: "app".into(),
        title: "app".into(),
        status: ProjectStatus::Active,
        icon: "folder".into(),
        description: None,
        blueprint_markdown: None,
        repo_folder: None,
        auto_commit: false,
        main_branch: None,
        pull_request_flow: false,
        task_key: "AP".into(),
        company_id: company.id,
        created_at: now,
        updated_at: now,
    };
    project.clone().into_active_model().insert(&database.conn).await.unwrap();
    let services = Services::new(ctx.clone(), std::sync::Arc::new(rekall_core::check::ScriptedCheckRunner::new()));
    World {
        tasks: TaskService::new(ctx.clone(), services.review.clone(), services.time_entries.clone()),
        conversion: TaskEpicConversionService::new(ctx.clone(), services.time_entries.clone()),
        dependencies: TaskDependencyService::new(ctx.clone(), RunQueueService::new(ctx.clone())),
        steps: services.steps.clone(),
        project_id: project.id,
        ctx,
    }
}

async fn task_in(world: &World, label: &str, kind: TaskKind, epic_id: Option<Id>) -> TaskView {
    let command = TaskCommand {
        label: Some(label.into()),
        title: Some(label.into()),
        description: Some(format!("About {label}")),
        status: Some(TaskStatus::InProgress),
        project_id: Some(world.project_id),
        kind: Some(kind),
        epic_id,
        ..TaskCommand::default()
    };
    world.tasks.create(command).await.unwrap()
}

#[tokio::test]
async fn the_task_keeps_its_title_status_and_description_as_an_epic() {
    let world = world().await;
    let task = task_in(&world, "handoff", TaskKind::Task, None).await;

    let epic = world.conversion.convert(task.id).await.unwrap();

    assert_eq!(epic.kind, TaskKind::Epic);
    assert_eq!(epic.code, task.code);
    assert_eq!(epic.status, TaskStatus::InProgress);
    assert_eq!(epic.description.as_deref(), Some("About handoff"));
}

#[tokio::test]
async fn the_preview_counts_what_the_conversion_deletes_and_the_conversion_deletes_it() {
    let world = world().await;
    let task = task_in(&world, "handoff", TaskKind::Task, None).await;
    world.steps.add(task.id, NewStep::titled("Wire the form", None)).await.unwrap();
    world.steps.add(task.id, NewStep::titled("Test it", None)).await.unwrap();

    let preview = world.conversion.preview(task.id).await.unwrap();
    world.conversion.convert(task.id).await.unwrap();

    assert_eq!(preview.steps, 2);
    assert!(world.steps.find_by_task(task.id).await.unwrap().is_empty());
    assert_eq!(world.conversion.preview(task.id).await.unwrap().steps, 0);
}

#[tokio::test]
async fn dependencies_on_both_sides_are_deleted() {
    let world = world().await;
    let task = task_in(&world, "handoff", TaskKind::Task, None).await;
    let before = task_in(&world, "schema", TaskKind::Task, None).await;
    let after = task_in(&world, "expiry", TaskKind::Task, None).await;
    world.dependencies.add(task.id, TaskDependencyCommand { depends_on_task_id: before.id }).await.unwrap();
    world.dependencies.add(after.id, TaskDependencyCommand { depends_on_task_id: task.id }).await.unwrap();

    let preview = world.conversion.preview(task.id).await.unwrap();
    world.conversion.convert(task.id).await.unwrap();

    assert_eq!((preview.waits_for, preview.needed_by), (1, 1));
    assert!(world.dependencies.list_of(task.id).await.unwrap().is_empty());
    assert!(world.dependencies.list_of(after.id).await.unwrap().is_empty());
}

#[tokio::test]
async fn the_task_leaves_the_epic_it_belonged_to() {
    let world = world().await;
    let parent = task_in(&world, "platform", TaskKind::Epic, None).await;
    let task = task_in(&world, "handoff", TaskKind::Task, Some(parent.id)).await;

    let preview = world.conversion.preview(task.id).await.unwrap();
    let epic = world.conversion.convert(task.id).await.unwrap();

    assert_eq!(preview.epic_code, Some(parent.code));
    assert_eq!(epic.epic_id, None);
}

#[tokio::test]
async fn an_epic_is_not_converted_again() {
    let world = world().await;
    let epic = task_in(&world, "platform", TaskKind::Epic, None).await;

    let refused = world.conversion.convert(epic.id).await;

    assert!(matches!(refused, Err(RekallError::Conflict(_))));
}

#[tokio::test]
async fn a_task_on_the_run_queue_is_not_converted() {
    let world = world().await;
    let task = task_in(&world, "handoff", TaskKind::Task, None).await;
    RunQueueService::new(world.ctx.clone()).add(task.id, Default::default()).await.unwrap();

    let refused = world.conversion.convert(task.id).await;

    assert!(world.conversion.preview(task.id).await.unwrap().queued);
    assert!(matches!(refused, Err(RekallError::Conflict(_))));
}
