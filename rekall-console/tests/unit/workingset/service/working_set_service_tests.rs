use std::sync::Arc;

use rekall_core::EventBus;
use rekall_kernel::Instant;
use rekall_store::{company, project, task, ProjectStatus};
use sea_orm::{ActiveModelTrait, IntoActiveModel};

use super::*;

struct World {
    service: WorkingSetService,
    vega: project::Model,
    atlas: project::Model,
    vega_tasks: Vec<task::Model>,
    atlas_task: task::Model,
    _database: rekall_store::Database,
}

async fn a_project(db: &sea_orm::DatabaseConnection, company_id: Id, label: &str, key: &str, now: Instant) -> project::Model {
    let project = project::Model {
        id: Id::random(), label: label.into(), title: label.to_uppercase(), status: ProjectStatus::Active, icon: "folder".into(),
        description: None, blueprint_markdown: None, repo_folder: None, auto_commit: false, main_branch: None, pull_request_flow: false, task_key: key.into(),
        company_id, created_at: now, updated_at: now,
    };
    project.clone().into_active_model().insert(db).await.unwrap();
    project
}

async fn a_task(db: &sea_orm::DatabaseConnection, project: &project::Model, label: &str, number: i32, now: Instant) -> task::Model {
    let mut task = task::Model::new(label.into(), label.to_uppercase(), project.id, now);
    task.number = number;
    task.clone().into_active_model().insert(db).await.unwrap();
    task
}

async fn world() -> World {
    let database = rekall_store::open_temporary().await.unwrap();
    let mut ctx = Ctx::new(database.conn.clone(), EventBus::new());
    let now = Instant::parse("2026-11-23T12:00:00Z").unwrap();
    ctx.clock = Arc::new(move || now);
    let db = &database.conn;
    let company = company::Model { id: Id::random(), name: "Acme".into(), description: None, created_at: now, updated_at: now };
    company.clone().into_active_model().insert(db).await.unwrap();
    let vega = a_project(db, company.id, "vega", "VG", now).await;
    let atlas = a_project(db, company.id, "atlas", "AT", now).await;
    let vega_tasks = vec![a_task(db, &vega, "alpha", 1, now).await, a_task(db, &vega, "beta", 2, now).await, a_task(db, &vega, "gamma", 3, now).await];
    let atlas_task = a_task(db, &atlas, "delta", 1, now).await;
    World { service: WorkingSetService::new(ctx), vega, atlas, vega_tasks, atlas_task, _database: database }
}

#[tokio::test]
async fn the_view_lists_open_tasks_in_order_with_their_project() {
    let world = world().await;
    world.service.open(world.vega_tasks[1].id).await.unwrap();
    world.service.open(world.atlas_task.id).await.unwrap();
    let view = world.service.open(world.vega_tasks[1].id).await.unwrap();

    assert_eq!(
        view.tasks,
        [OpenTaskView { task_id: world.vega_tasks[1].id, project_id: world.vega.id }, OpenTaskView { task_id: world.atlas_task.id, project_id: world.atlas.id }]
    );
}

#[tokio::test]
async fn opening_a_task_that_does_not_exist_is_not_found() {
    let world = world().await;

    let refused = world.service.open(Id::random()).await.unwrap_err();

    assert!(matches!(refused, RekallError::NotFound(_)), "{refused:?}");
}

#[tokio::test]
async fn closing_answers_the_neighbour_in_the_same_project() {
    let world = world().await;
    for task in &world.vega_tasks {
        world.service.open(task.id).await.unwrap();
    }
    world.service.open(world.atlas_task.id).await.unwrap();

    let landing = world.service.close(world.vega_tasks[1].id).await.unwrap();

    assert_eq!(landing, PlaceView { project_id: world.vega.id, task_id: Some(world.vega_tasks[2].id) });
    assert_eq!(world.service.view().await.unwrap().tasks.len(), 3);
}

#[tokio::test]
async fn closing_the_last_open_task_of_a_project_answers_its_task_list() {
    let world = world().await;
    world.service.open(world.atlas_task.id).await.unwrap();

    let landing = world.service.close(world.atlas_task.id).await.unwrap();

    assert_eq!(landing, PlaceView { project_id: world.atlas.id, task_id: None });
}

#[tokio::test]
async fn a_project_that_remembered_the_closed_task_remembers_where_it_landed() {
    let world = world().await;
    world.service.open(world.vega_tasks[0].id).await.unwrap();
    world.service.open(world.vega_tasks[1].id).await.unwrap();
    world.service.remember_place(world.vega.id, Some(world.vega_tasks[0].id)).await.unwrap();

    world.service.close(world.vega_tasks[0].id).await.unwrap();

    let places = world.service.view().await.unwrap().places;
    assert_eq!(places, [PlaceView { project_id: world.vega.id, task_id: Some(world.vega_tasks[1].id) }]);
}

#[tokio::test]
async fn a_place_in_another_project_is_refused() {
    let world = world().await;

    let refused = world.service.remember_place(world.vega.id, Some(world.atlas_task.id)).await.unwrap_err();

    assert!(refused.is_illegal_argument(), "{refused:?}");
    assert!(world.service.view().await.unwrap().places.is_empty());
}

#[tokio::test]
async fn a_place_in_a_project_is_remembered_for_the_task_list_too() {
    let world = world().await;

    let place = world.service.remember_place(world.vega.id, None).await.unwrap();

    assert_eq!(world.service.view().await.unwrap().places, [place]);
}
