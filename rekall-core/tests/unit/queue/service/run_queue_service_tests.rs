use std::sync::Arc;

use rekall_store::{company, project, task, ProjectStatus};

use super::*;
use crate::EventBus;

struct World {
    service: RunQueueService,
    now: Instant,
    first: Id,
    second: Id,
}

async fn world() -> World {
    let database = rekall_store::open_in_memory().await.unwrap();
    let now = Instant::now();
    let mut ctx = Ctx::new(database.conn.clone(), EventBus::new());
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
    World { service: RunQueueService::new(ctx), now, first: first.id, second: second.id }
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
