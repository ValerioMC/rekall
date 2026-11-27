use rekall_kernel::Instant;
use rekall_store::{company, project, task, task_step, ProjectStatus};
use sea_orm::{ActiveModelTrait, IntoActiveModel};

use super::*;
use crate::EventBus;

struct World {
    service: TaskStepService,
    events: EventBus,
    task_id: Id,
}

async fn world(states: &[TaskStepState]) -> World {
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
    for (position, state) in states.iter().enumerate() {
        let mut step = task_step::Model::new(task.id, format!("Step {position}"), position as i32, now);
        step.mark_state(*state);
        step.into_active_model().insert(&database.conn).await.unwrap();
    }
    let events = EventBus::new();
    let ctx = Ctx::new(database.conn.clone(), events.clone());
    let service = TaskStepService::new(ctx.clone(), TimeEntryService::new(ctx));
    World { service, events, task_id: task.id }
}

async fn states_of(world: &World) -> Vec<TaskStepState> {
    world.service.find_by_task(world.task_id).await.unwrap().into_iter().map(|step| step.state).collect()
}

fn announced_checklists(feed: &mut tokio::sync::broadcast::Receiver<crate::events::PublishedEvent>) -> usize {
    std::iter::from_fn(|| feed.try_recv().ok()).filter(|event| event.downcast::<StepStreamEvent>().is_some()).count()
}

#[tokio::test]
async fn releasing_a_tasks_running_steps_reopens_each_one_and_leaves_the_others() {
    use TaskStepState::{Claimed, Done, Draft, Open, Running};
    let world = world(&[Done, Claimed, Running, Open, Running, Draft]).await;
    let mut feed = world.events.subscribe();

    world.service.release_all_running(world.task_id).await.unwrap();

    assert_eq!(states_of(&world).await, [Done, Claimed, Open, Open, Open, Draft]);
    assert_eq!(announced_checklists(&mut feed), 1);
}

#[tokio::test]
async fn releasing_a_task_with_nothing_running_announces_nothing() {
    let world = world(&[TaskStepState::Claimed, TaskStepState::Open]).await;
    let mut feed = world.events.subscribe();

    world.service.release_all_running(world.task_id).await.unwrap();

    assert_eq!(announced_checklists(&mut feed), 0);
}
