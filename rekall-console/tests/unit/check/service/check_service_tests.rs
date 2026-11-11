use rekall_core::{EventBus, Services};
use rekall_kernel::Instant;
use rekall_store::{company, project, ProjectStatus, TaskKind};

use super::*;
use crate::catalog::{TaskCommand, TaskService};

struct World {
    service: CheckService,
    tasks: TaskService,
    steps: rekall_core::step::TaskStepService,
    database: rekall_store::Database,
    project_id: Id,
    task_id: Id,
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
        auto_commit: false, main_branch: None,
        company_id: company.id,
        created_at: now,
        updated_at: now,
    };
    project.clone().into_active_model().insert(&database.conn).await.unwrap();
    let services = Services::new(ctx.clone(), std::sync::Arc::new(rekall_core::check::ScriptedCheckRunner::new()));
    let tasks = TaskService::new(ctx.clone(), services.review.clone(), services.time_entries.clone());
    let task = tasks
        .create(TaskCommand { label: Some("export".into()), title: Some("Export".into()), project_id: Some(project.id), kind: Some(TaskKind::Task), ..TaskCommand::default() })
        .await
        .unwrap();
    World { service: CheckService::new(ctx), tasks, steps: services.steps.clone(), database, project_id: project.id, task_id: task.id }
}

fn check(label: &str) -> CheckCommand {
    CheckCommand { label: Some(label.into()), command: Some("cargo test --workspace".into()), folder: None, timeout_seconds: Some(600) }
}

fn ids(views: &[CheckView]) -> Vec<String> {
    views.iter().map(|view| view.id.to_string()).collect()
}

#[tokio::test]
async fn a_project_check_is_listed_with_the_next_position_of_its_scope() {
    let world = world().await;
    let first = world.service.create_for_project(world.project_id, check("backend tests")).await.unwrap();
    let second = world.service.create_for_project(world.project_id, check("ui lint")).await.unwrap();

    assert_eq!((first.position, second.position), (0, 1));
    assert_eq!(first.step_id, None);
    assert_eq!(world.service.list(world.project_id).await.unwrap(), vec![first, second]);
}

#[tokio::test]
async fn a_step_check_takes_the_project_of_its_task_and_lists_after_the_project_ones() {
    let world = world().await;
    let step = world.steps.add(world.task_id, Some("Wire the form"), None).await.unwrap();
    let on_step = world.service.create_for_step(step.id, check("form test")).await.unwrap();
    let on_project = world.service.create_for_project(world.project_id, check("backend tests")).await.unwrap();

    assert_eq!(on_step.project_id, world.project_id);
    assert_eq!(on_step.step_id, Some(step.id));
    assert_eq!(on_step.position, 0);
    assert_eq!(world.service.list(world.project_id).await.unwrap(), vec![on_project, on_step]);
}

#[tokio::test]
async fn a_check_is_edited_field_by_field_and_a_blank_folder_clears_it() {
    let world = world().await;
    let created = world.service.create_for_project(world.project_id, CheckCommand { folder: Some("rekall-ui".into()), ..check("ui test") }).await.unwrap();
    assert_eq!(created.folder.as_deref(), Some("rekall-ui"));

    let renamed = world.service.update(created.id, CheckCommand { label: Some("ui tests".into()), ..CheckCommand::default() }).await.unwrap();
    assert_eq!((renamed.label.as_str(), renamed.folder.as_deref(), renamed.timeout_seconds), ("ui tests", Some("rekall-ui"), 600));

    let cleared = world.service.update(created.id, CheckCommand { folder: Some("  ".into()), ..CheckCommand::default() }).await.unwrap();
    assert_eq!(cleared.folder, None);
}

#[tokio::test]
async fn a_scope_is_put_in_order_and_only_that_scope() {
    let world = world().await;
    let step = world.steps.add(world.task_id, Some("Wire the form"), None).await.unwrap();
    let first = world.service.create_for_project(world.project_id, check("one")).await.unwrap();
    let second = world.service.create_for_project(world.project_id, check("two")).await.unwrap();
    let on_step = world.service.create_for_step(step.id, check("form")).await.unwrap();

    let ordered = world.service.reorder_for_project(world.project_id, CheckOrderCommand { check_ids: Some(ids(&[second.clone(), first.clone()])) }).await.unwrap();

    assert_eq!(ordered.iter().map(|view| view.label.as_str()).collect::<Vec<_>>(), ["two", "one"]);
    let listed = world.service.list(world.project_id).await.unwrap();
    assert_eq!(listed.iter().map(|view| view.label.as_str()).collect::<Vec<_>>(), ["two", "one", "form"]);
    assert_eq!(listed[2].position, on_step.position);
}

#[tokio::test]
async fn an_order_that_misses_or_repeats_a_check_is_refused() {
    let world = world().await;
    let first = world.service.create_for_project(world.project_id, check("one")).await.unwrap();
    world.service.create_for_project(world.project_id, check("two")).await.unwrap();

    let missing = world.service.reorder_for_project(world.project_id, CheckOrderCommand { check_ids: Some(ids(&[first.clone()])) }).await;
    let repeated = world.service.reorder_for_project(world.project_id, CheckOrderCommand { check_ids: Some(ids(&[first.clone(), first])) }).await;

    assert!(missing.unwrap_err().is_illegal_argument());
    assert!(repeated.unwrap_err().is_illegal_argument());
}

#[tokio::test]
async fn a_blank_or_too_long_text_and_a_timeout_out_of_range_are_client_mistakes() {
    let world = world().await;
    let blank_command = world.service.create_for_project(world.project_id, CheckCommand { command: Some("   ".into()), ..check("tests") }).await;
    let long_label = world.service.create_for_project(world.project_id, check(&"x".repeat(101))).await;
    let long_command = world.service.create_for_project(world.project_id, CheckCommand { command: Some("x".repeat(2_001)), ..check("tests") }).await;
    let no_timeout = world.service.create_for_project(world.project_id, CheckCommand { timeout_seconds: Some(0), ..check("tests") }).await;
    let missing_timeout = world.service.create_for_project(world.project_id, CheckCommand { timeout_seconds: None, ..check("tests") }).await;

    for refused in [blank_command, long_label, long_command, no_timeout, missing_timeout] {
        assert!(refused.unwrap_err().is_illegal_argument());
    }
    assert!(world.service.list(world.project_id).await.unwrap().is_empty());
}

#[tokio::test]
async fn a_check_of_a_missing_project_or_step_is_not_found() {
    let world = world().await;
    assert!(matches!(world.service.create_for_project(Id::random(), check("tests")).await.unwrap_err(), RekallError::NotFound(_)));
    assert!(matches!(world.service.create_for_step(Id::random(), check("tests")).await.unwrap_err(), RekallError::NotFound(_)));
    assert!(matches!(world.service.delete(Id::random()).await.unwrap_err(), RekallError::NotFound(_)));
}

#[tokio::test]
async fn deleting_a_check_removes_only_that_check() {
    let world = world().await;
    let first = world.service.create_for_project(world.project_id, check("one")).await.unwrap();
    let second = world.service.create_for_project(world.project_id, check("two")).await.unwrap();

    world.service.delete(first.id).await.unwrap();

    assert_eq!(world.service.list(world.project_id).await.unwrap(), vec![second]);
}

#[tokio::test]
async fn deleting_the_task_removes_the_checks_of_its_steps_and_keeps_the_projects() {
    let world = world().await;
    let step = world.steps.add(world.task_id, Some("Wire the form"), None).await.unwrap();
    world.service.create_for_step(step.id, check("form")).await.unwrap();
    let on_project = world.service.create_for_project(world.project_id, check("backend tests")).await.unwrap();

    world.tasks.delete(world.task_id).await.unwrap();

    assert_eq!(world.service.list(world.project_id).await.unwrap(), vec![on_project]);
}

#[tokio::test]
async fn deleting_the_project_removes_its_checks() {
    let world = world().await;
    world.service.create_for_project(world.project_id, check("backend tests")).await.unwrap();

    project::Entity::delete_by_id(world.project_id).exec(&world.database.conn).await.unwrap();

    assert!(done_check::Entity::find().all(&world.database.conn).await.unwrap().is_empty());
}

#[tokio::test]
async fn the_results_of_a_task_are_listed_latest_first_with_the_label_of_their_check() {
    let world = world().await;
    let step = world.steps.add(world.task_id, Some("Wire the form"), None).await.unwrap();
    let on_project = world.service.create_for_project(world.project_id, check("backend tests")).await.unwrap();
    let on_step = world.service.create_for_step(step.id, check("form test")).await.unwrap();
    let ran = |check: &CheckView, step_id: Option<Id>, exit_code: Option<i32>, seconds: i64| rekall_store::done_check_result::Model {
        id: Id::random(),
        check_id: check.id,
        task_id: world.task_id,
        step_id,
        passed: exit_code == Some(0),
        exit_code,
        output_tail: "tail".into(),
        ran_at: Instant::now().plus_seconds(seconds),
        duration_ms: 40,
    };
    for result in [ran(&on_project, Some(step.id), Some(0), 0), ran(&on_step, Some(step.id), None, 5)] {
        result.into_active_model().insert(&world.database.conn).await.unwrap();
    }

    let results = world.service.results_of_task(world.task_id).await.unwrap();

    assert_eq!(results.iter().map(|result| result.label.as_str()).collect::<Vec<_>>(), ["form test", "backend tests"]);
    assert_eq!((results[0].passed, results[0].timed_out), (false, true));
    assert_eq!((results[1].passed, results[1].timed_out), (true, false));
}

#[tokio::test]
async fn the_results_of_a_task_that_does_not_exist_are_not_found() {
    let world = world().await;
    let refused = world.service.results_of_task(Id::random()).await.unwrap_err();

    assert!(matches!(refused, RekallError::NotFound(_)), "{refused:?}");
}
