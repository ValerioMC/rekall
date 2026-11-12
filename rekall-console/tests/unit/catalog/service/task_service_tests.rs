use rekall_kernel::Instant;
use rekall_store::{company, task_pull_request, ProjectStatus, TaskKind, TaskStepState};
use rekall_core::step::{NewStep, StepEdit};
use rekall_core::task::TaskContentChangedEvent;
use rekall_core::{EventBus, Services};
use sea_orm::{ActiveModelTrait, IntoActiveModel};

use super::*;

struct World {
    db: sea_orm::DatabaseConnection,
    events: EventBus,
    service: TaskService,
    steps: rekall_core::step::TaskStepService,
    project_id: Id,
    other_project_id: Id,
}

async fn world() -> World {
    let database = rekall_store::open_in_memory().await.unwrap();
    let events = EventBus::new();
    let ctx = Ctx::new(database.conn.clone(), events.clone());
    let now = Instant::now();
    let company = company::Model { id: Id::random(), name: "Acme".into(), description: None, created_at: now, updated_at: now };
    company.clone().into_active_model().insert(&database.conn).await.unwrap();
    let project_id = insert_project(&database.conn, company.id, "app").await;
    let other_project_id = insert_project(&database.conn, company.id, "site").await;
    let services = Services::new(ctx.clone(), std::sync::Arc::new(rekall_core::check::ScriptedCheckRunner::new()));
    let service = TaskService::new(ctx, services.review.clone(), services.time_entries.clone());
    World { db: database.conn.clone(), events, service, steps: services.steps.clone(), project_id, other_project_id }
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
        auto_commit: false, main_branch: None, pull_request_flow: false,
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
    world.steps.add(task.id, NewStep::titled("Wire the form", None)).await.unwrap();
    let refused = world.service.update(task.id, command("checkout", world.project_id, TaskKind::Epic, None)).await;
    assert!(refused.is_err());
}

#[tokio::test]
async fn an_epic_takes_no_step() {
    let world = world().await;
    let epic = world.service.create(command("checkout", world.project_id, TaskKind::Epic, None)).await.unwrap();
    assert!(world.steps.add(epic.id, NewStep::titled("Wire the form", None)).await.is_err());
}

#[tokio::test]
async fn deleting_an_epic_leaves_its_tasks_on_their_own() {
    let world = world().await;
    let epic = world.service.create(command("checkout", world.project_id, TaskKind::Epic, None)).await.unwrap();
    let task = world.service.create(command("pay", world.project_id, TaskKind::Task, Some(epic.id))).await.unwrap();
    world.service.delete(epic.id).await.unwrap();
    assert_eq!(world.service.get(task.id).await.unwrap().epic_id, None);
}

fn done_command(label: &str, project_id: Id) -> TaskCommand {
    TaskCommand { status: Some(TaskStatus::Done), ..command(label, project_id, TaskKind::Task, None) }
}

#[tokio::test]
async fn putting_a_task_with_steps_to_done_accepts_every_step_but_the_drafts() {
    let world = world().await;
    let task = world.service.create(command("checkout", world.project_id, TaskKind::Task, None)).await.unwrap();
    let worked = world.steps.add(task.id, NewStep::titled("Wire the form", None)).await.unwrap();
    world.steps.add(task.id, NewStep::titled("Maybe later", None)).await.unwrap();
    world.steps.edit(worked.id, StepEdit { draft: Some(false), ..StepEdit::default() }).await.unwrap();

    let saved = world.service.update(task.id, done_command("checkout", world.project_id)).await.unwrap();

    let states: Vec<_> = world.steps.find_by_task(task.id).await.unwrap().into_iter().map(|step| step.state).collect();
    assert_eq!(states, [TaskStepState::Done, TaskStepState::Draft]);
    assert_eq!(saved.steps_done, 1);
}

#[tokio::test]
async fn putting_a_task_without_steps_to_done_accepts_its_review_line() {
    let world = world().await;
    let task = world.service.create(command("checkout", world.project_id, TaskKind::Task, None)).await.unwrap();

    let saved = world.service.update(task.id, done_command("checkout", world.project_id)).await.unwrap();

    assert_eq!(saved.review_state, TaskStepState::Done);
    assert!(saved.accepted_at.is_some());
}

#[tokio::test]
async fn putting_an_epic_to_done_leaves_its_review_line_alone() {
    let world = world().await;
    let epic = world.service.create(command("checkout", world.project_id, TaskKind::Epic, None)).await.unwrap();
    let done = TaskCommand { status: Some(TaskStatus::Done), ..command("checkout", world.project_id, TaskKind::Epic, None) };

    let saved = world.service.update(epic.id, done).await.unwrap();

    assert_eq!(saved.review_state, TaskStepState::Open);
}

fn announced_tasks(feed: &mut tokio::sync::broadcast::Receiver<rekall_core::PublishedEvent>) -> Vec<Id> {
    let mut tasks = Vec::new();
    while let Ok(event) = feed.try_recv() {
        if let Some(changed) = event.downcast::<TaskContentChangedEvent>() {
            tasks.push(changed.task_id);
        }
    }
    tasks
}

#[tokio::test]
async fn a_created_task_is_announced_for_its_task_manager() {
    let world = world().await;
    let mut feed = world.events.subscribe();

    let task = world.service.create(command("pay", world.project_id, TaskKind::Task, None)).await.unwrap();

    assert_eq!(announced_tasks(&mut feed), [task.id]);
}

#[tokio::test]
async fn a_task_imported_with_its_issue_is_not_announced_for_a_push_back() {
    let world = world().await;
    let mut feed = world.events.subscribe();
    let imported = TaskCommand { tracker_issue_key: Some("ENG-1".into()), tracker_issue_url: Some("https://jira.example.com/browse/ENG-1".into()), ..command("eng-1", world.project_id, TaskKind::Task, None) };

    let task = world.service.create(imported).await.unwrap();

    assert!(announced_tasks(&mut feed).is_empty());
    assert_eq!(task.tracker_issue_key.as_deref(), Some("ENG-1"));
}

#[tokio::test]
async fn an_edit_is_announced_only_when_the_title_or_the_description_changed() {
    let world = world().await;
    let task = world.service.create(command("pay", world.project_id, TaskKind::Task, None)).await.unwrap();
    let mut feed = world.events.subscribe();
    let same_content = TaskCommand { status: Some(TaskStatus::InProgress), ..command("pay", world.project_id, TaskKind::Task, None) };
    let retitled = TaskCommand { title: Some("Pay the invoice".into()), ..command("pay", world.project_id, TaskKind::Task, None) };

    world.service.update(task.id, same_content).await.unwrap();
    assert!(announced_tasks(&mut feed).is_empty());
    world.service.update(task.id, retitled).await.unwrap();

    assert_eq!(announced_tasks(&mut feed), [task.id]);
}

#[tokio::test]
async fn an_edit_keeps_the_issue_the_task_was_written_to() {
    let world = world().await;
    let imported = TaskCommand {
        tracker_issue_key: Some("12".into()),
        tracker_issue_url: Some("https://github.com/acme/app/issues/12".into()),
        ..command("pay", world.project_id, TaskKind::Task, None)
    };
    let task = world.service.create(imported).await.unwrap();

    let edited = world.service.update(task.id, TaskCommand { title: Some("Pay".into()), ..command("pay", world.project_id, TaskKind::Task, None) }).await.unwrap();

    assert_eq!(edited.tracker_issue_key.as_deref(), Some("12"));
}

#[tokio::test]
async fn the_task_view_carries_the_pull_request_opened_for_its_branch() {
    let world = world().await;
    let task = world.service.create(command("pay", world.project_id, TaskKind::Task, None)).await.unwrap();
    assert_eq!(task.pull_request_number, None);
    let now = Instant::now();
    let opened = task_pull_request::Model { task_id: task.id, number: Some(5), url: Some("https://github.com/acme/app/pull/5".into()), failure: None, merged_at: None, created_at: now, updated_at: now };
    rekall_store::repository::task_pull_request::save(&world.db, opened).await.unwrap();

    let view = world.service.get(task.id).await.unwrap();

    assert_eq!((view.pull_request_number, view.pull_request_url.as_deref(), view.pull_request_failure), (Some(5), Some("https://github.com/acme/app/pull/5"), None));
}

#[tokio::test]
async fn accepting_a_task_with_steps_accepts_every_claimed_step_once_all_are_claimed() {
    let world = world().await;
    let task = world.service.create(command("checkout", world.project_id, TaskKind::Task, None)).await.unwrap();
    for title in ["Wire the form", "Pay"] {
        let step = world.steps.add(task.id, NewStep::titled(title, None)).await.unwrap();
        world.steps.edit(step.id, StepEdit { draft: Some(false), ..StepEdit::default() }).await.unwrap();
    }
    world.steps.transition(Some("app"), "checkout", Some("Wire the form"), TaskStepState::Claimed).await.unwrap();

    let early = world.service.review(task.id, TaskStepState::Done).await;
    world.steps.transition(Some("app"), "checkout", Some("Pay"), TaskStepState::Claimed).await.unwrap();
    world.service.review(task.id, TaskStepState::Done).await.unwrap();

    assert!(matches!(early, Err(RekallError::IllegalArgument(_))));
    let states: Vec<TaskStepState> = world.steps.find_by_task(task.id).await.unwrap().into_iter().map(|step| step.state).collect();
    assert_eq!(states, [TaskStepState::Done, TaskStepState::Done]);
    assert!(matches!(world.service.review(task.id, TaskStepState::Done).await, Err(RekallError::IllegalArgument(_))));
}

#[tokio::test]
async fn a_deleted_task_announces_the_issue_it_held_and_one_without_an_issue_announces_nothing() {
    let world = world().await;
    let held = world.service.create(TaskCommand { tracker_issue_key: Some("7".into()), tracker_issue_url: Some("https://github.com/acme/app/issues/7".into()), ..command("pay", world.project_id, TaskKind::Task, None) }).await.unwrap();
    let bare = world.service.create(command("ship", world.project_id, TaskKind::Task, None)).await.unwrap();
    let mut feed = world.events.subscribe();

    world.service.delete(held.id).await.unwrap();
    world.service.delete(bare.id).await.unwrap();

    let mut deleted = Vec::new();
    while let Ok(event) = feed.try_recv() {
        if let Some(event) = event.downcast::<rekall_core::task::TaskDeletedEvent>() {
            deleted.push(event.clone());
        }
    }
    assert_eq!(deleted, [rekall_core::task::TaskDeletedEvent { project_id: world.project_id, issue_key: Some("7".into()) }]);
}
