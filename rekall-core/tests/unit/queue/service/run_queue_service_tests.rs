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
    third: Id,
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
        auto_commit: false, main_branch: None, pull_request_flow: false, task_key: String::new(),
        company_id: company.id,
        created_at: now,
        updated_at: now,
    };
    project.clone().into_active_model().insert(&database.conn).await.unwrap();
    let first = task::Model::new("report".into(), "Report".into(), project.id, now);
    first.clone().into_active_model().insert(&database.conn).await.unwrap();
    let second = task::Model::new("export".into(), "Export".into(), project.id, now);
    second.clone().into_active_model().insert(&database.conn).await.unwrap();
    let third = task::Model::new("archive".into(), "Archive".into(), project.id, now);
    third.clone().into_active_model().insert(&database.conn).await.unwrap();
    World { service: RunQueueService::new(ctx), events, db: database.conn.clone(), now, first: first.id, second: second.id, third: third.id }
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

async fn depend(world: &World, task: Id, on: Id) {
    rekall_store::task_dependency::Model::new(task, on, world.now).into_active_model().insert(&world.db).await.unwrap();
}

async fn review(world: &World, task: Id, state: TaskStepState) {
    let mut row = rekall_store::task::Entity::find_by_id(task).one(&world.db).await.unwrap().unwrap();
    row.mark_review_state(state);
    row.into_active_model().reset_all().update(&world.db).await.unwrap();
}

fn ready_task(turn: NextTurn) -> Id {
    match turn {
        NextTurn::Ready(item) => item.task_id,
        other => panic!("expected a task ready to start, got {other:?}"),
    }
}

#[tokio::test]
async fn the_next_turn_goes_to_the_first_waiting_task_when_none_depends_on_another() {
    let world = world().await;
    world.service.add(world.first).await.unwrap();
    world.service.add(world.second).await.unwrap();

    assert_eq!(ready_task(world.service.next_turn(&HashSet::new()).await.unwrap()), world.first);
}

#[tokio::test]
async fn a_chain_is_ordered_by_its_dependencies_whatever_the_order_it_was_queued_in() {
    let world = world().await;
    depend(&world, world.second, world.first).await;
    depend(&world, world.third, world.second).await;
    world.service.add(world.third).await.unwrap();
    world.service.add(world.second).await.unwrap();
    let view = world.service.add(world.first).await.unwrap();

    assert_eq!(ready_task(world.service.next_turn(&HashSet::new()).await.unwrap()), world.first);

    world.service.mark_item(view.items[2].id, RunQueueItemState::Finished, None).await.unwrap();
    review(&world, world.first, TaskStepState::Done).await;
    assert_eq!(ready_task(world.service.next_turn(&HashSet::new()).await.unwrap()), world.second);
}

#[tokio::test]
async fn a_dependency_claimed_but_not_accepted_keeps_the_task_waiting() {
    let world = world().await;
    depend(&world, world.first, world.second).await;
    world.service.add(world.first).await.unwrap();
    review(&world, world.second, TaskStepState::Claimed).await;

    let turn = world.service.next_turn(&HashSet::new()).await.unwrap();

    let NextTurn::Blocked(blocked) = turn else { panic!("expected a blocked queue, got {turn:?}") };
    assert_eq!(blocked[0].reason, "Waiting for task:EXPORT (claimed, waiting for your review).");
}

#[tokio::test]
async fn an_accepted_dependency_lets_the_task_start() {
    let world = world().await;
    depend(&world, world.first, world.second).await;
    world.service.add(world.first).await.unwrap();
    review(&world, world.second, TaskStepState::Done).await;

    assert_eq!(ready_task(world.service.next_turn(&HashSet::new()).await.unwrap()), world.first);
}

#[tokio::test]
async fn an_item_names_the_unaccepted_tasks_it_waits_for_and_whether_they_are_queued() {
    let world = world().await;
    depend(&world, world.first, world.second).await;
    depend(&world, world.first, world.third).await;
    world.service.add(world.third).await.unwrap();

    let view = world.service.add(world.first).await.unwrap();

    let mut waits: Vec<(&str, bool)> = view.items[1].waiting_for.iter().map(|wait| (wait.title.as_str(), wait.in_queue)).collect();
    waits.sort();
    assert_eq!(waits, vec![("Archive", true), ("Export", false)]);
}

#[tokio::test]
async fn a_queue_where_nothing_may_start_stops_and_says_why_on_each_waiting_item() {
    let world = world().await;
    depend(&world, world.first, world.second).await;
    world.service.add_and_arm(world.first).await.unwrap();
    let NextTurn::Blocked(blocked) = world.service.next_turn(&HashSet::new()).await.unwrap() else { panic!("expected a blocked queue") };

    world.service.stop_blocked(&blocked).await.unwrap();

    let view = world.service.view().await.unwrap();
    assert_eq!(view.state, RunQueueState::Idle);
    assert_eq!(view.items[0].state, RunQueueItemState::Queued);
    assert_eq!(view.items[0].detail.as_deref(), Some("Waiting for task:EXPORT."));
}

#[tokio::test]
async fn the_queue_works_one_task_at_a_time_until_told_otherwise() {
    let world = world().await;

    assert_eq!(world.service.view().await.unwrap().parallel_sessions, 1);
    assert_eq!(world.service.snapshot().await.unwrap().parallel_sessions, 1);
}

#[tokio::test]
async fn parallel_sessions_are_kept_between_one_and_four() {
    let world = world().await;

    let view = world.service.update_settings(None, false, None, None, rekall_store::SessionKind::Terminal, 4).await.unwrap();
    let too_many = world.service.update_settings(None, false, None, None, rekall_store::SessionKind::Terminal, 5).await.unwrap_err();
    let none = world.service.update_settings(None, false, None, None, rekall_store::SessionKind::Terminal, 0).await.unwrap_err();

    assert_eq!(view.parallel_sessions, 4);
    assert!(matches!(too_many, RekallError::IllegalArgument(_)) && matches!(none, RekallError::IllegalArgument(_)));
    assert_eq!(world.service.snapshot().await.unwrap().parallel_sessions, 4);
}

#[tokio::test]
async fn a_parked_item_is_passed_over_and_reported_with_its_reason() {
    let world = world().await;
    world.service.add(world.first).await.unwrap();
    let view = world.service.add(world.second).await.unwrap();
    world.service.mark_item(view.items[0].id, RunQueueItemState::Queued, Some("git could not make a worktree.")).await.unwrap();

    let ready = world.service.next_turn(&HashSet::from([view.items[0].id])).await.unwrap();
    let only_parked = world.service.next_turn(&HashSet::from([view.items[0].id, view.items[1].id])).await.unwrap();

    assert_eq!(ready_task(ready), world.second);
    let NextTurn::Blocked(blocked) = only_parked else { panic!("expected a blocked queue") };
    assert_eq!(blocked[0].reason, "git could not make a worktree.");
}
