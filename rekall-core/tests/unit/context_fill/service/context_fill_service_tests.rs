use rekall_kernel::Instant;
use rekall_store::{company, project, task, task_step, ProjectStatus};
use sea_orm::{ActiveModelTrait, IntoActiveModel};

use super::*;
use crate::EventBus;

struct World {
    service: ContextFillService,
    events: EventBus,
    task_id: Id,
    step_id: Id,
}

async fn world() -> World {
    let database = rekall_store::open_in_memory().await.unwrap();
    let now = Instant::now();
    let company = company::Model { id: Id::random(), name: "Acme".into(), description: None, created_at: now, updated_at: now };
    company.clone().into_active_model().insert(&database.conn).await.unwrap();
    let project = project::Model {
        id: Id::random(), label: "vega".into(), title: "Vega".into(), status: ProjectStatus::Active, icon: "folder".into(),
        description: None, blueprint_markdown: None, repo_folder: None, auto_commit: false, main_branch: None, pull_request_flow: false, task_key: String::new(), company_id: company.id,
        created_at: now, updated_at: now,
    };
    project.clone().into_active_model().insert(&database.conn).await.unwrap();
    let task = task::Model::new("alpha".into(), "Alpha".into(), project.id, now);
    task.clone().into_active_model().insert(&database.conn).await.unwrap();
    let step = task_step::Model::new(task.id, "First".into(), 0, now);
    step.clone().into_active_model().insert(&database.conn).await.unwrap();
    let events = EventBus::new();
    let service = ContextFillService::new(Ctx::new(database.conn.clone(), events.clone()));
    World { service, events, task_id: task.id, step_id: step.id }
}

fn fill(used_tokens: u64) -> ContextFill {
    ContextFill { used_tokens, window_tokens: 200_000 }
}

#[tokio::test]
async fn a_turn_on_a_step_writes_the_steps_row_and_the_tasks() {
    let world = world().await;

    world.service.record(world.task_id, Some(world.step_id), fill(50_000)).await.unwrap();

    let fills = world.service.for_task(world.task_id).await.unwrap();
    let on = |step_id: Option<Id>| fills.iter().find(|view| view.step_id == step_id).unwrap().percent;
    assert_eq!(fills.len(), 2);
    assert_eq!((on(None), on(Some(world.step_id))), (25, 25));
}

#[tokio::test]
async fn a_turn_with_no_step_writes_only_the_tasks_row() {
    let world = world().await;

    world.service.record(world.task_id, None, fill(100_000)).await.unwrap();

    let fills = world.service.for_task(world.task_id).await.unwrap();
    assert_eq!(fills.iter().map(|view| (view.step_id, view.used_tokens, view.window_tokens)).collect::<Vec<_>>(), [(None, 100_000, 200_000)]);
}

#[tokio::test]
async fn one_event_carrying_the_tasks_whole_list_is_published_after_the_commit() {
    let world = world().await;
    let mut feed = world.events.subscribe();

    world.service.record(world.task_id, Some(world.step_id), fill(50_000)).await.unwrap();

    let mut announced = Vec::new();
    while let Ok(event) = feed.try_recv() {
        if let Some(event) = event.downcast::<ContextFillStreamEvent>() {
            announced.push((event.task_id, event.fills.len()));
        }
    }
    assert_eq!(announced, [(world.task_id, 2)]);
}
