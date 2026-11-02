use std::sync::Arc;

use rekall_store::{company, project, task, task_step, ProjectStatus, TaskStepState};

use super::*;
use crate::EventBus;

struct World {
    service: RunQueueService,
    events: EventBus,
    db: sea_orm::DatabaseConnection,
    now: Instant,
    first: Id,
    second: Id,
}

async fn world() -> World {
    let database = rekall_store::open_in_memory().await.unwrap();
    let now = Instant::now();
    let events = EventBus::new();
    let mut ctx = Ctx::new(database.conn.clone(), events.clone());
    ctx.clock = Arc::new(move || now);
    let company = company::Model { id: Id::random(), name: "Acme".into(), description: None, created_at: now, updated_at: now };
    company.clone().into_active_model().insert(&database.conn).await.unwrap();
    let project = project::Model {
        id: Id::random(),
        label: "vega".into(),
        title: "Vega".into(),
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
    let first = task::Model::new("report".into(), "Report".into(), project.id, now);
    first.clone().into_active_model().insert(&database.conn).await.unwrap();
    let second = task::Model::new("export".into(), "Export".into(), project.id, now);
    second.clone().into_active_model().insert(&database.conn).await.unwrap();
    World { service: RunQueueService::new(ctx), events, db: database.conn.clone(), now, first: first.id, second: second.id }
}

#[tokio::test]
async fn an_idle_queue_starts_with_the_task_it_is_given() {
    let world = world().await;

    let view = world.service.add_and_arm(world.first).await.unwrap();

    assert_eq!(view.state, RunQueueState::Running);
    assert_eq!(view.items.len(), 1);
    assert_eq!(view.items[0].state, RunQueueItemState::Queued);
}

#[tokio::test]
async fn a_running_queue_only_appends_the_task() {
    let world = world().await;
    world.service.add_and_arm(world.first).await.unwrap();

    let view = world.service.add_and_arm(world.second).await.unwrap();

    assert_eq!(view.state, RunQueueState::Running);
    assert_eq!(view.items.iter().map(|item| item.task_id).collect::<Vec<_>>(), vec![world.first, world.second]);
}

#[tokio::test]
async fn a_scheduled_queue_keeps_its_start_time() {
    let world = world().await;
    let later = world.now.plus_minutes(30);
    world.service.add(world.first).await.unwrap();
    world.service.start(Some(later)).await.unwrap();

    let view = world.service.add_and_arm(world.second).await.unwrap();

    assert_eq!(view.state, RunQueueState::Scheduled);
    assert_eq!(view.start_at, Some(later));
    assert_eq!(view.items.len(), 2);
}

#[tokio::test]
async fn a_task_already_queued_is_refused() {
    let world = world().await;
    world.service.add_and_arm(world.first).await.unwrap();

    let refused = world.service.add_and_arm(world.first).await.unwrap_err();

    assert!(matches!(refused, RekallError::Conflict(_)));
}

#[tokio::test]
async fn finishing_with_a_task_still_waiting_keeps_the_queue_running() {
    let world = world().await;
    world.service.add_and_arm(world.first).await.unwrap();

    world.service.finish().await.unwrap();

    assert_eq!(world.service.view().await.unwrap().state, RunQueueState::Running);
}

#[tokio::test]
async fn finishing_with_nothing_waiting_returns_the_queue_to_idle() {
    let world = world().await;
    let view = world.service.add_and_arm(world.first).await.unwrap();
    world.service.mark_item(view.items[0].id, RunQueueItemState::Finished, None).await.unwrap();

    world.service.finish().await.unwrap();

    assert_eq!(world.service.view().await.unwrap().state, RunQueueState::Idle);
}

async fn step(world: &World, task: Id, title: &str, position: i32, state: TaskStepState) {
    let mut step = task_step::Model::new(task, title.into(), position, world.now);
    step.state = state;
    step.into_active_model().insert(&world.db).await.unwrap();
}

#[tokio::test]
async fn an_item_carries_its_tasks_steps_and_the_one_being_worked() {
    let world = world().await;
    step(&world, world.first, "Model", 0, TaskStepState::Claimed).await;
    step(&world, world.first, "View", 1, TaskStepState::Open).await;
    step(&world, world.first, "Idea", 2, TaskStepState::Draft).await;

    let view = world.service.add(world.first).await.unwrap();

    let steps = &view.items[0].steps;
    assert_eq!(steps.iter().map(|step| step.title.as_str()).collect::<Vec<_>>(), vec!["Model", "View"]);
    assert_eq!(steps.iter().map(|step| step.current).collect::<Vec<_>>(), vec![false, true]);
}

#[tokio::test]
async fn a_task_without_a_checklist_has_no_steps_on_its_item() {
    let world = world().await;

    let view = world.service.add(world.first).await.unwrap();

    assert!(view.items[0].steps.is_empty());
}

fn announced_queues(events: &mut tokio::sync::broadcast::Receiver<crate::PublishedEvent>) -> usize {
    let mut count = 0;
    while let Ok(event) = events.try_recv() {
        if event.downcast::<RunQueueFrame>().is_some() {
            count += 1;
        }
    }
    count
}

#[tokio::test]
async fn a_step_move_on_a_listed_task_announces_the_queue_again() {
    let world = world().await;
    world.service.add(world.first).await.unwrap();
    let mut events = world.events.subscribe();

    world.service.refresh_for_task(world.first).await.unwrap();

    assert_eq!(announced_queues(&mut events), 1);
}

#[tokio::test]
async fn a_step_move_on_a_task_outside_the_queue_announces_nothing() {
    let world = world().await;
    world.service.add(world.first).await.unwrap();
    let mut events = world.events.subscribe();

    world.service.refresh_for_task(world.second).await.unwrap();

    assert_eq!(announced_queues(&mut events), 0);
}
