use std::path::Path;

use rekall_kernel::Instant;
use rekall_store::{company, project, task, task_step, ProjectStatus, TaskStepState};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use tempfile::TempDir;

use super::*;
use crate::check::ScriptedCheckRunner;
use crate::EventBus;

struct World {
    service: CheckExecutionService,
    runner: Arc<ScriptedCheckRunner>,
    database: rekall_store::Database,
    project_folder: TempDir,
    project_id: Id,
    step_id: Id,
    other_step_id: Id,
    task_id: Id,
}

const STEPLESS_TASK: &str = "quick";

async fn world(runner: ScriptedCheckRunner) -> World {
    world_with_folder(runner, true).await
}

async fn world_with_folder(runner: ScriptedCheckRunner, has_folder: bool) -> World {
    let database = rekall_store::open_in_memory().await.unwrap();
    let now = Instant::now();
    let project_folder = tempfile::tempdir().unwrap();
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
        repo_folder: has_folder.then(|| project_folder.path().to_string_lossy().into_owned()),
        auto_commit: false, main_branch: None, pull_request_flow: false,
        company_id: company.id,
        created_at: now,
        updated_at: now,
    };
    project.clone().into_active_model().insert(&database.conn).await.unwrap();
    let task = task::Model::new("export".into(), "Export".into(), project.id, now);
    task.clone().into_active_model().insert(&database.conn).await.unwrap();
    let stepless = task::Model::new(STEPLESS_TASK.into(), "Quick".into(), project.id, now);
    stepless.clone().into_active_model().insert(&database.conn).await.unwrap();
    let mut step = task_step::Model::new(task.id, "Wire the form".into(), 0, now);
    step.state = TaskStepState::Running;
    step.clone().into_active_model().insert(&database.conn).await.unwrap();
    let mut other_step = task_step::Model::new(task.id, "Wire the list".into(), 1, now);
    other_step.state = TaskStepState::Draft;
    other_step.clone().into_active_model().insert(&database.conn).await.unwrap();
    let runner = Arc::new(runner);
    let service = CheckExecutionService::new(Ctx::new(database.conn.clone(), EventBus::new()), runner.clone());
    World { service, runner, database, project_folder, project_id: project.id, step_id: step.id, other_step_id: other_step.id, task_id: task.id }
}

impl World {
    async fn add_check(&self, label: &str, command: &str, step_id: Option<Id>, position: i32, folder: Option<&str>) {
        let now = Instant::now();
        let check = done_check::Model {
            id: Id::random(),
            project_id: self.project_id,
            step_id,
            label: label.into(),
            command: command.into(),
            folder: folder.map(str::to_string),
            timeout_seconds: 30,
            position,
            created_at: now,
            updated_at: now,
        };
        check.into_active_model().insert(&self.database.conn).await.unwrap();
    }

    async fn kept_results(&self) -> Vec<done_check_result::Model> {
        done_check_result::Entity::find()
            .filter(done_check_result::Column::TaskId.eq(self.task_id))
            .order_by_asc(done_check_result::Column::RanAt)
            .all(&self.database.conn)
            .await
            .unwrap()
    }

    async fn database_results_of(&self, task_label: &str) -> Vec<done_check_result::Model> {
        let task = task::Entity::find().filter(task::Column::Label.eq(task_label)).one(&self.database.conn).await.unwrap().unwrap();
        done_check_result::Entity::find().filter(done_check_result::Column::TaskId.eq(task.id)).all(&self.database.conn).await.unwrap()
    }

    fn commands_run(&self) -> Vec<String> {
        self.runner.runs().into_iter().map(|(_, command, _)| command).collect()
    }
}

fn labels(runs: &[CheckRan]) -> Vec<&str> {
    runs.iter().map(|run| run.label.as_str()).collect()
}

#[tokio::test]
async fn a_step_without_checks_reports_none_and_runs_nothing() {
    let world = world(ScriptedCheckRunner::new()).await;

    let report = world.service.run_all(world.step_id).await.unwrap();

    assert_eq!(report, CheckReport::NoChecks);
    assert!(world.commands_run().is_empty());
    assert!(world.kept_results().await.is_empty());
}

#[tokio::test]
async fn checks_that_all_pass_run_in_order_the_project_first_and_each_result_is_kept() {
    let world = world(ScriptedCheckRunner::new()).await;
    world.add_check("form test", "pnpm test", Some(world.step_id), 0, None).await;
    world.add_check("backend tests", "cargo test", None, 0, None).await;
    world.add_check("ui lint", "pnpm lint", None, 1, None).await;

    let report = world.service.run_all(world.step_id).await.unwrap();

    let CheckReport::Passed { runs } = report else { panic!("expected Passed, got {report:?}") };
    assert_eq!(labels(&runs), vec!["backend tests", "ui lint", "form test"]);
    assert_eq!(world.commands_run(), vec!["cargo test", "pnpm lint", "pnpm test"]);
    let kept = world.kept_results().await;
    assert_eq!(kept.len(), 3);
    assert!(kept.iter().all(|result| result.passed && result.exit_code == Some(0) && result.step_id == Some(world.step_id)));
}

#[tokio::test]
async fn a_failing_check_stops_the_run_and_the_checks_after_it_are_not_started() {
    let runner = ScriptedCheckRunner::new().answering("pnpm lint", CheckOutcome::Exited(2), "lint: 3 errors");
    let world = world(runner).await;
    world.add_check("backend tests", "cargo test", None, 0, None).await;
    world.add_check("ui lint", "pnpm lint", None, 1, None).await;
    world.add_check("ui typecheck", "pnpm typecheck", None, 2, None).await;

    let report = world.service.run_all(world.step_id).await.unwrap();

    assert_eq!(report, CheckReport::Failed { label: "ui lint".into(), outcome: CheckOutcome::Exited(2), output_tail: "lint: 3 errors".into() });
    assert_eq!(world.commands_run(), vec!["cargo test", "pnpm lint"]);
    let kept = world.kept_results().await;
    assert_eq!(kept.iter().map(|result| (result.passed, result.exit_code)).collect::<Vec<_>>(), vec![(true, Some(0)), (false, Some(2))]);
    assert_eq!(kept[1].output_tail, "lint: 3 errors");
}

#[tokio::test]
async fn a_check_that_times_out_fails_and_keeps_no_exit_code() {
    let runner = ScriptedCheckRunner::new().answering("cargo test", CheckOutcome::TimedOut, "still compiling");
    let world = world(runner).await;
    world.add_check("backend tests", "cargo test", None, 0, None).await;

    let report = world.service.run_all(world.step_id).await.unwrap();

    assert!(matches!(report, CheckReport::Failed { outcome: CheckOutcome::TimedOut, .. }));
    let kept = world.kept_results().await;
    assert_eq!((kept[0].passed, kept[0].exit_code), (false, None));
}

#[tokio::test]
async fn the_checks_of_another_step_are_not_run() {
    let world = world(ScriptedCheckRunner::new()).await;
    world.add_check("list test", "pnpm test list", Some(world.other_step_id), 0, None).await;
    world.add_check("form test", "pnpm test form", Some(world.step_id), 0, None).await;

    world.service.run_all(world.step_id).await.unwrap();

    assert_eq!(world.commands_run(), vec!["pnpm test form"]);
}

#[tokio::test]
async fn a_check_runs_in_the_project_folder_or_the_folder_inside_it_with_its_timeout() {
    let world = world(ScriptedCheckRunner::new()).await;
    world.add_check("backend tests", "cargo test", None, 0, None).await;
    world.add_check("ui lint", "pnpm lint", None, 1, Some("rekall-ui")).await;

    world.service.run_all(world.step_id).await.unwrap();

    let runs = world.runner.runs();
    let root: &Path = world.project_folder.path();
    assert_eq!(runs[0].0, root);
    assert_eq!(runs[1].0, root.join("rekall-ui"));
    assert_eq!(runs[0].2, Duration::from_secs(30));
}

#[tokio::test]
async fn a_folder_leaving_the_project_is_refused_before_any_command_runs() {
    let world = world(ScriptedCheckRunner::new()).await;
    world.add_check("backend tests", "cargo test", None, 0, None).await;
    world.add_check("elsewhere", "ls", None, 1, Some("../outside")).await;

    let failure = world.service.run_all(world.step_id).await.unwrap_err();

    assert!(failure.is_illegal_argument());
    assert!(world.commands_run().is_empty());
}

#[tokio::test]
async fn a_project_without_a_folder_cannot_run_its_checks() {
    let world = world_with_folder(ScriptedCheckRunner::new(), false).await;
    world.add_check("backend tests", "cargo test", None, 0, None).await;

    let failure = world.service.run_all(world.step_id).await.unwrap_err();

    assert!(failure.is_illegal_argument());
}

#[tokio::test]
async fn a_project_without_a_folder_and_without_checks_still_reports_none() {
    let world = world_with_folder(ScriptedCheckRunner::new(), false).await;

    assert_eq!(world.service.run_all(world.step_id).await.unwrap(), CheckReport::NoChecks);
}

#[tokio::test]
async fn claiming_a_step_by_its_labels_runs_the_checks_of_that_step() {
    let world = world(ScriptedCheckRunner::new()).await;
    world.add_check("backend tests", "cargo test", None, 0, None).await;
    world.add_check("form test", "pnpm test form", Some(world.step_id), 0, None).await;

    let report = world.service.run_for_step_claim(Some("app"), "export", Some("1")).await.unwrap();

    let CheckReport::Passed { runs } = report else { panic!("expected Passed, got {report:?}") };
    assert_eq!(labels(&runs), vec!["backend tests", "form test"]);
    assert!(world.kept_results().await.iter().all(|result| result.step_id == Some(world.step_id)));
}

#[tokio::test]
async fn a_step_that_cannot_be_claimed_runs_no_check() {
    let world = world(ScriptedCheckRunner::new()).await;
    world.add_check("backend tests", "cargo test", None, 0, None).await;

    let draft = world.service.run_for_step_claim(Some("app"), "export", Some("Wire the list")).await.unwrap();

    assert_eq!(draft, CheckReport::NoChecks);
    assert!(world.commands_run().is_empty());
}

#[tokio::test]
async fn claiming_a_step_that_does_not_exist_is_refused_like_any_other_move() {
    let world = world(ScriptedCheckRunner::new()).await;

    let failure = world.service.run_for_step_claim(Some("app"), "export", Some("9")).await.unwrap_err();

    assert!(failure.is_illegal_argument());
}

#[tokio::test]
async fn the_wrapup_of_a_task_with_no_steps_runs_the_project_checks_and_keeps_them_without_a_step() {
    let world = world(ScriptedCheckRunner::new()).await;
    world.add_check("backend tests", "cargo test", None, 0, None).await;
    world.add_check("form test", "pnpm test form", Some(world.step_id), 0, None).await;

    let report = world.service.run_for_wrapup_claim(Some("app"), STEPLESS_TASK).await.unwrap();

    let CheckReport::Passed { runs } = report else { panic!("expected Passed, got {report:?}") };
    assert_eq!(labels(&runs), vec!["backend tests"]);
    let kept = world.database_results_of(STEPLESS_TASK).await;
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].step_id, None);
}

#[tokio::test]
async fn the_wrapup_of_a_task_with_steps_runs_no_check() {
    let world = world(ScriptedCheckRunner::new()).await;
    world.add_check("backend tests", "cargo test", None, 0, None).await;

    let report = world.service.run_for_wrapup_claim(Some("app"), "export").await.unwrap();

    assert_eq!(report, CheckReport::NoChecks);
    assert!(world.commands_run().is_empty());
}

#[tokio::test]
async fn a_failing_check_refuses_the_wrapup_claim_and_keeps_the_run() {
    let runner = ScriptedCheckRunner::new().answering("cargo test", CheckOutcome::Exited(101), "1 test failed");
    let world = world(runner).await;
    world.add_check("backend tests", "cargo test", None, 0, None).await;

    let report = world.service.run_for_wrapup_claim(Some("app"), STEPLESS_TASK).await.unwrap();

    assert!(matches!(report, CheckReport::Failed { ref label, .. } if label == "backend tests"));
    assert!(!world.database_results_of(STEPLESS_TASK).await[0].passed);
}
