use rekall_core::EventBus;
use rekall_kernel::Instant;
use rekall_store::{company, document, document_task, project, step_visit, DocumentContextMode, ProjectStatus, TaskKind};
use sea_orm::{ActiveModelTrait, IntoActiveModel};

use super::*;
use crate::catalog::{TaskCommand, TaskService};

struct World {
    service: PlanAdherenceService,
    database: rekall_store::Database,
    project_id: Id,
    task_id: Id,
    tasks: TaskService,
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
        company_id: company.id,
        created_at: now,
        updated_at: now,
    };
    project.clone().into_active_model().insert(&database.conn).await.unwrap();
    let services = rekall_core::Services::new(ctx.clone(), std::sync::Arc::new(rekall_core::check::ScriptedCheckRunner::new()));
    let tasks = TaskService::new(ctx.clone(), services.review.clone(), services.time_entries.clone());
    let task_id = new_task(&tasks, project.id, "export").await;
    World { service: PlanAdherenceService::new(ctx), database, project_id: project.id, task_id, tasks }
}

async fn new_task(tasks: &TaskService, project_id: Id, label: &str) -> Id {
    let command = TaskCommand { label: Some(label.into()), title: Some(label.into()), project_id: Some(project_id), kind: Some(TaskKind::Task), ..TaskCommand::default() };
    tasks.create(command).await.unwrap().id
}

async fn add_step(world: &World, task_id: Id, position: i32, state: TaskStepState, sent_back_times: usize) -> task_step::Model {
    let mut step = task_step::Model::new(task_id, format!("Step {position}"), position, Instant::now());
    send_back(&mut step, sent_back_times);
    step.state = state;
    step.clone().into_active_model().insert(&world.database.conn).await.unwrap();
    step
}

fn send_back(step: &mut task_step::Model, times: usize) {
    for _ in 0..times {
        step.state = TaskStepState::Claimed;
        step.send_back(Instant::now()).unwrap();
    }
}

async fn add_note(world: &World, task_id: Id, title: &str) {
    let now = Instant::now();
    let note = document::Model {
        id: Id::random(),
        title: title.into(),
        kind: "notes".into(),
        body_markdown: String::new(),
        source_path: None,
        context_mode: DocumentContextMode::Full,
        scope_company_id: None,
        scope_project_id: None,
        created_at: now,
        updated_at: now,
    };
    note.clone().into_active_model().insert(&world.database.conn).await.unwrap();
    document_task::Model { document_id: note.id, task_id, position: 0 }.into_active_model().insert(&world.database.conn).await.unwrap();
}

async fn enter(world: &World, step: &task_step::Model, ordinal: i32) {
    step_visit::Model::enter(step.task_id, step.id, ordinal, Instant::now()).into_active_model().insert(&world.database.conn).await.unwrap();
}

#[tokio::test]
async fn a_step_sent_back_twice_counts_three_passes_and_the_others_one_each() {
    let world = world().await;
    add_step(&world, world.task_id, 0, TaskStepState::Done, 0).await;
    add_step(&world, world.task_id, 1, TaskStepState::Done, 2).await;
    add_step(&world, world.task_id, 2, TaskStepState::Done, 0).await;

    let view = world.service.of_task(world.task_id).await.unwrap();

    assert_eq!(
        view.counts,
        AdherenceCounts { steps_total: 3, steps_done: 3, steps_claimed: 0, accepted_first_time: 2, sent_back: 1, passes_total: 5 }
    );
    assert_eq!(view.counts.first_time_share(), Some(2.0 / 3.0));
}

#[tokio::test]
async fn a_step_sent_back_and_not_yet_started_again_has_only_its_earlier_passes() {
    let world = world().await;
    add_step(&world, world.task_id, 0, TaskStepState::Open, 1).await;
    add_step(&world, world.task_id, 1, TaskStepState::Claimed, 0).await;
    add_step(&world, world.task_id, 2, TaskStepState::Draft, 0).await;

    let counts = world.service.of_task(world.task_id).await.unwrap().counts;

    assert_eq!((counts.steps_total, counts.steps_claimed, counts.sent_back, counts.passes_total), (2, 1, 1, 2));
    assert_eq!(counts.first_time_share(), None);
}

#[tokio::test]
async fn a_deviation_note_is_counted_with_the_step_its_title_names() {
    let world = world().await;
    add_note(&world, world.task_id, "step-2-changes.md").await;
    add_note(&world, world.task_id, "solution-overview.md").await;

    let view = world.service.of_task(world.task_id).await.unwrap();

    assert_eq!(view.deviation_steps, vec![2]);
}

#[tokio::test]
async fn a_task_with_no_step_gives_a_view_of_zeros() {
    let world = world().await;

    let view = world.service.of_task(world.task_id).await.unwrap();

    assert_eq!(view, PlanAdherenceView { counts: AdherenceCounts::default(), deviation_steps: vec![], visits: vec![] });
    assert_eq!(view.counts.first_time_share(), None);
}

#[tokio::test]
async fn a_flow_shows_the_visits_of_each_step_and_which_spent_their_budget() {
    let world = world().await;
    let first = add_step(&world, world.task_id, 0, TaskStepState::Done, 0).await;
    let mut fix = add_step(&world, world.task_id, 1, TaskStepState::Running, 0).await;
    fix.max_visits = Some(2);
    fix.clone().into_active_model().reset_all().update(&world.database.conn).await.unwrap();
    add_step(&world, world.task_id, 2, TaskStepState::Open, 0).await;
    enter(&world, &first, 1).await;
    enter(&world, &fix, 2).await;
    enter(&world, &fix, 3).await;

    let visits = world.service.of_task(world.task_id).await.unwrap().visits;

    assert_eq!(
        visits,
        vec![
            StepVisitsView { step_number: 1, title: "Step 0".into(), visits: 1, max_visits: None, budget_spent: false },
            StepVisitsView { step_number: 2, title: "Step 1".into(), visits: 2, max_visits: Some(2), budget_spent: true },
        ]
    );
}

#[tokio::test]
async fn a_task_that_does_not_exist_is_not_found() {
    let world = world().await;

    assert!(world.service.of_task(Id::random()).await.is_err());
    assert!(world.service.of_project(Id::random()).await.is_err());
}

#[tokio::test]
async fn a_project_sums_its_tasks_and_lists_at_most_five_with_steps_sent_back_most_first() {
    let world = world().await;
    add_step(&world, world.task_id, 0, TaskStepState::Done, 1).await;
    let other = new_task(&world.tasks, world.project_id, "import").await;
    add_step(&world, other, 0, TaskStepState::Done, 0).await;
    add_step(&world, other, 1, TaskStepState::Done, 3).await;
    add_step(&world, other, 2, TaskStepState::Done, 1).await;
    new_task(&world.tasks, world.project_id, "untouched").await;
    add_note(&world, other, "step-2-changes.md").await;

    let view = world.service.of_project(world.project_id).await.unwrap();

    assert_eq!(view.counts.steps_total, 4);
    assert_eq!((view.counts.accepted_first_time, view.counts.sent_back, view.counts.passes_total), (1, 3, 9));
    assert_eq!(view.deviation_notes, 1);
    let listed: Vec<(&str, usize)> = view.most_sent_back.iter().map(|task| (task.label.as_str(), task.sent_back)).collect();
    assert_eq!(listed, vec![("import", 2), ("export", 1)]);
}

#[tokio::test]
async fn only_five_tasks_are_listed() {
    let world = world().await;
    for index in 0..7 {
        let task_id = new_task(&world.tasks, world.project_id, &format!("task-{index}")).await;
        add_step(&world, task_id, 0, TaskStepState::Done, 1).await;
    }

    let view = world.service.of_project(world.project_id).await.unwrap();

    assert_eq!(view.most_sent_back.len(), 5);
}
