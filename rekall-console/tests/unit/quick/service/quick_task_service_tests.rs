use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use rekall_core::claude::{SessionKind, TerminalEnded, TerminalHost, TerminalOpening, TerminalView};
use rekall_core::{Ctx, EventBus, Services};
use rekall_kernel::Instant;
use rekall_store::{company, project, ProjectStatus, TaskStepState};
use sea_orm::{ActiveModelTrait, IntoActiveModel};
use tokio::sync::broadcast;

use super::*;

/// Opens every terminal unless told not to, and remembers each opening it was asked for.
struct FakeHost {
    refuse_with: Option<&'static str>,
    openings: Mutex<Vec<TerminalOpening>>,
}

#[async_trait]
impl TerminalHost for FakeHost {
    async fn open(&self, opening: TerminalOpening) -> Result<TerminalView> {
        if let Some(reason) = self.refuse_with {
            return Err(RekallError::illegal(reason));
        }
        let view = view_on(&opening.target);
        self.openings.lock().unwrap().push(opening);
        Ok(view)
    }

    fn list(&self) -> Vec<TerminalView> {
        Vec::new()
    }

    fn get(&self, _id: Id) -> Option<TerminalView> {
        None
    }

    async fn close(&self, _id: Id, _reason: &str) -> Option<TerminalView> {
        None
    }

    fn subscribe_ended(&self) -> broadcast::Receiver<TerminalEnded> {
        broadcast::channel(1).0.subscribe()
    }

    fn has_live_for(&self, _task_id: Id) -> bool {
        false
    }

    fn work_session_for(&self, _task_id: Id) -> Option<TerminalView> {
        None
    }

    async fn renew(&self, id: Id) -> Result<TerminalView> {
        Err(RekallError::conflict(format!("The fake host does not renew {id}.")))
    }
}

fn view_on(target: &TerminalTarget) -> TerminalView {
    let task_id = match target {
        TerminalTarget::Task(task_id) => Some(*task_id),
        TerminalTarget::Project(_) => None,
    };
    let now = Instant::now();
    TerminalView {
        id: Id::random(),
        project_id: Id::random(),
        task_id,
        step_id: None,
        anchors: "project:app".into(),
        working_dir: "/work/app".into(),
        project_label: "app".into(),
        task_label: None,
        task_title: None,
        mode: TerminalMode::Work,
        skip_permissions: false,
        model: None,
        effort: None,
        kind: SessionKind::Terminal,
        live: true,
        awaiting_permission: false,
        started_at: now,
        last_activity_at: now,
        connections: Vec::new(),
    }
}

struct World {
    service: QuickTaskService,
    tasks: TaskService,
    host: Arc<FakeHost>,
    project_id: Id,
}

async fn world(refuse_with: Option<&'static str>) -> World {
    let database = rekall_store::open_in_memory().await.unwrap();
    let ctx = Ctx::new(database.conn.clone(), EventBus::new());
    let now = Instant::now();
    let company = company::Model { id: Id::random(), name: "Acme".into(), description: None, created_at: now, updated_at: now };
    company.clone().into_active_model().insert(&database.conn).await.unwrap();
    let project = project::Model {
        id: Id::random(),
        label: "app".into(),
        title: "App".into(),
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
    let services = Services::new(ctx.clone(), Arc::new(rekall_core::check::ScriptedCheckRunner::new()));
    let tasks = TaskService::new(ctx.clone(), services.review.clone(), services.time_entries.clone());
    let host = Arc::new(FakeHost { refuse_with, openings: Mutex::new(Vec::new()) });
    let service = QuickTaskService::new(ctx, tasks.clone(), TerminalService::new(host.clone()));
    World { service, tasks, host, project_id: project.id }
}

fn command(project_id: Id, text: &str) -> QuickTaskCommand {
    QuickTaskCommand { project_id, text: text.into(), skip_permissions: true, model: Some("sonnet".into()), effort: None, kind: SessionKind::Chat }
}

#[tokio::test]
async fn the_text_becomes_a_quick_task_titled_by_its_first_line_with_a_session_opened_on_it() {
    let world = world(None).await;
    let started = world.service.create(command(world.project_id, "Fix the typo in README\nIt says recieve.")).await.unwrap();
    assert!(started.task.quick);
    assert!(!started.task.auto_accepted);
    assert_eq!(started.task.title, "Fix the typo in README");
    assert_eq!(started.task.label, "fix-the-typo-in-readme");
    assert_eq!(started.task.description.as_deref(), Some("Fix the typo in README\nIt says recieve."));
    assert_eq!(started.task.review_state, TaskStepState::Open);
    let openings = world.host.openings.lock().unwrap();
    assert_eq!(openings.len(), 1);
    assert_eq!(openings[0].target, TerminalTarget::Task(started.task.id));
    assert!(openings[0].skip_permissions);
    assert_eq!(openings[0].model.as_deref(), Some("sonnet"));
    assert_eq!(openings[0].kind, SessionKind::Chat);
}

#[tokio::test]
async fn a_blank_text_is_refused_and_nothing_is_created() {
    let world = world(None).await;
    let refusal = world.service.create(command(world.project_id, "  \n ")).await.unwrap_err();
    assert!(matches!(refusal, RekallError::IllegalArgument(_)), "{refusal}");
    assert!(world.tasks.list(Some(world.project_id)).await.unwrap().is_empty());
}

#[tokio::test]
async fn a_second_quick_task_is_refused_while_the_first_is_still_being_worked() {
    let world = world(None).await;
    world.service.create(command(world.project_id, "First change")).await.unwrap();
    let refusal = world.service.create(command(world.project_id, "Second change")).await.unwrap_err();
    assert!(matches!(refusal, RekallError::Conflict(_)), "{refusal}");
    assert_eq!(world.tasks.list(Some(world.project_id)).await.unwrap().len(), 1);
}

#[tokio::test]
async fn the_same_request_twice_gets_a_numbered_label_once_the_first_is_out_of_the_way() {
    let world = world(None).await;
    let first = world.service.create(command(world.project_id, "Fix the typo")).await.unwrap();
    world.tasks.review(first.task.id, TaskStepState::Done, None).await.unwrap();
    let second = world.service.create(command(world.project_id, "Fix the typo")).await.unwrap();
    assert_eq!(second.task.label, "fix-the-typo-2");
}

#[tokio::test]
async fn a_session_that_cannot_open_takes_the_task_away_with_it() {
    let world = world(Some("Set this project's folder on its page before opening a terminal here.")).await;
    let refusal = world.service.create(command(world.project_id, "Fix the typo")).await.unwrap_err();
    assert!(refusal.to_string().contains("Set this project's folder"), "{refusal}");
    assert!(world.tasks.list(Some(world.project_id)).await.unwrap().is_empty());
}
