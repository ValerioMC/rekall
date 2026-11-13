use std::sync::Mutex;

use async_trait::async_trait;
use rekall_core::claude::{SessionKind, TerminalEnded, TerminalMode};
use rekall_core::worktree::{ScriptedWorktreeHost, WorktreeLayout};
use rekall_core::{Ctx, EventBus};
use rekall_kernel::Instant;
use rekall_store::{company, external_connection, project, project_connection, task, ConnectionKind, ProjectStatus};
use sea_orm::{ActiveModelTrait, IntoActiveModel};
use tokio::sync::broadcast;

use super::*;

/// Opens every terminal it is asked for and remembers each opening, so a test sees what reached it.
#[derive(Default)]
struct FakeHost {
    openings: Mutex<Vec<TerminalOpening>>,
    live: Mutex<Vec<TerminalView>>,
    ended: Mutex<Option<broadcast::Sender<TerminalEnded>>>,
}

#[async_trait]
impl TerminalHost for FakeHost {
    async fn open(&self, opening: TerminalOpening) -> Result<TerminalView> {
        let view = view_on(&opening.target);
        self.openings.lock().unwrap().push(opening);
        self.live.lock().unwrap().push(view.clone());
        Ok(view)
    }

    fn list(&self) -> Vec<TerminalView> {
        self.live.lock().unwrap().clone()
    }

    fn get(&self, id: Id) -> Option<TerminalView> {
        self.live.lock().unwrap().iter().find(|view| view.id == id).cloned()
    }

    async fn close(&self, id: Id, _reason: &str) -> Option<TerminalView> {
        let mut live = self.live.lock().unwrap();
        let position = live.iter().position(|view| view.id == id)?;
        Some(live.remove(position))
    }

    fn subscribe_ended(&self) -> broadcast::Receiver<TerminalEnded> {
        self.ended.lock().unwrap().get_or_insert_with(|| broadcast::channel(8).0).subscribe()
    }

    fn has_live_for(&self, task_id: Id) -> bool {
        self.live.lock().unwrap().iter().any(|view| view.task_id == Some(task_id))
    }

    fn work_session_for(&self, task_id: Id) -> Option<TerminalView> {
        self.live.lock().unwrap().iter().find(|view| view.task_id == Some(task_id)).cloned()
    }

    async fn renew(&self, id: Id) -> Result<TerminalView> {
        Err(RekallError::conflict(format!("The fake host does not renew {id}.")))
    }
}

fn view_on(target: &TerminalTarget) -> TerminalView {
    let (project_id, task_id) = match target {
        TerminalTarget::Task(task_id) => (Id::random(), Some(*task_id)),
        TerminalTarget::Project(project_id) => (*project_id, None),
    };
    let now = Instant::now();
    TerminalView {
        id: Id::random(),
        project_id,
        task_id,
        step_id: None,
        anchors: "project:vega".into(),
        working_dir: "/work/vega".into(),
        project_label: "vega".into(),
        task_code: None,
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

fn command(target: TerminalTarget, mode: TerminalMode) -> OpenTerminalCommand {
    OpenTerminalCommand { target, step_id: None, skip_permissions: true, model: Some("sonnet".into()), effort: None, mode, kind: SessionKind::Terminal }
}

struct Seed {
    db: sea_orm::DatabaseConnection,
    project_id: Id,
    worktree_host: Arc<ScriptedWorktreeHost>,
    _root: tempfile::TempDir,
}

impl Seed {
    async fn task(&self, label: &str) -> Id {
        let task = task::Model::new(label.into(), label.into(), self.project_id, Instant::now());
        task.clone().into_active_model().insert(&self.db).await.unwrap();
        task.id
    }

    async fn quick_task(&self, label: &str) -> Id {
        let mut task = task::Model::new(label.into(), label.into(), self.project_id, Instant::now());
        task.quick = true;
        task.clone().into_active_model().insert(&self.db).await.unwrap();
        task.id
    }

    async fn keep_tasks_on_branches(&self) {
        let now = Instant::now();
        let connection = external_connection::Model::new("github".into(), "GitHub".into(), ConnectionKind::Github, Some("https://github.com".into()), now);
        connection.clone().into_active_model().insert(&self.db).await.unwrap();
        let mut binding = project_connection::Model::new(self.project_id, connection.id, "github".into(), Some("acme/vega".into()), now);
        binding.task_manager = true;
        binding.into_active_model().insert(&self.db).await.unwrap();
    }
}

async fn service() -> (TerminalService, Arc<FakeHost>, Seed) {
    let database = rekall_store::open_in_memory().await.unwrap();
    let now = Instant::now();
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
        repo_folder: Some("/work/vega".into()),
        auto_commit: false, main_branch: None, pull_request_flow: false, task_key: String::new(),
        company_id: company.id,
        created_at: now,
        updated_at: now,
    };
    project.clone().into_active_model().insert(&database.conn).await.unwrap();
    let root = tempfile::tempdir().unwrap();
    let worktree_host = Arc::new(ScriptedWorktreeHost::new());
    let worktrees = TaskWorktreeService::new(Ctx::new(database.conn.clone(), EventBus::new()), worktree_host.clone(), WorktreeLayout::new(root.path().join("worktrees")));
    let host = Arc::new(FakeHost::default());
    (TerminalService::new(host.clone(), worktrees), host, Seed { db: database.conn.clone(), project_id: project.id, worktree_host, _root: root })
}

#[tokio::test]
async fn a_terminal_on_a_task_is_opened_with_everything_the_command_asked_for() {
    let (service, host, seed) = service().await;
    let task_id = seed.task("report").await;

    let opened = service.open(command(TerminalTarget::Task(task_id), TerminalMode::Plan)).await.unwrap();

    assert_eq!(opened.task_id, Some(task_id));
    let openings = host.openings.lock().unwrap();
    assert_eq!(openings.len(), 1);
    assert_eq!(openings[0].mode, TerminalMode::Plan);
    assert!(openings[0].skip_permissions);
    assert_eq!(openings[0].model.as_deref(), Some("sonnet"));
}

#[tokio::test]
async fn a_project_terminal_only_does_the_work_of_a_project_and_the_host_is_never_asked_otherwise() {
    let (service, host, _) = service().await;

    let refused = service.open(command(TerminalTarget::Project(Id::random()), TerminalMode::Work)).await;
    let inventory = service.open(command(TerminalTarget::Project(Id::random()), TerminalMode::Inventory)).await;

    assert!(matches!(refused, Err(RekallError::IllegalArgument(_))));
    assert!(inventory.is_ok());
    assert_eq!(host.openings.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn an_unknown_terminal_is_not_found_on_read_and_on_close() {
    let (service, _, _) = service().await;

    assert!(matches!(service.get(Id::random()), Err(RekallError::NotFound(_))));
    assert!(matches!(service.close(Id::random()).await, Err(RekallError::NotFound(_))));
}

#[tokio::test]
async fn closing_a_live_terminal_takes_it_off_the_list() {
    let (service, _, seed) = service().await;
    let opened = service.open(command(TerminalTarget::Task(seed.task("report").await), TerminalMode::Work)).await.unwrap();

    service.close(opened.id).await.unwrap();

    assert!(service.list().is_empty());
}

#[tokio::test]
async fn the_kind_of_session_the_command_asked_for_reaches_the_host() {
    let (service, host, seed) = service().await;
    let mut chat = command(TerminalTarget::Task(seed.task("chat").await), TerminalMode::Work);
    chat.kind = SessionKind::Chat;

    service.open(chat).await.unwrap();
    service.open(command(TerminalTarget::Task(seed.task("terminal").await), TerminalMode::Work)).await.unwrap();

    let kinds: Vec<SessionKind> = host.openings.lock().unwrap().iter().map(|opening| opening.kind).collect();
    assert_eq!(kinds, vec![SessionKind::Chat, SessionKind::Terminal]);
}

#[tokio::test]
async fn work_on_a_task_of_a_project_with_a_task_manager_starts_on_the_tasks_own_branch() {
    let (service, _, seed) = service().await;
    seed.keep_tasks_on_branches().await;
    let report = seed.task("report").await;

    service.open(command(TerminalTarget::Task(report), TerminalMode::Work)).await.unwrap();

    assert_eq!(seed.worktree_host.branches(), ["rekall/report"]);
}

#[tokio::test]
async fn a_session_that_does_not_work_the_task_or_a_project_without_a_task_manager_makes_no_branch() {
    let (service, _, seed) = service().await;
    let report = seed.task("report").await;

    service.open(command(TerminalTarget::Task(report), TerminalMode::Work)).await.unwrap();
    seed.keep_tasks_on_branches().await;
    service.open(command(TerminalTarget::Task(report), TerminalMode::Plan)).await.unwrap();
    service.open(command(TerminalTarget::Task(seed.quick_task("typo").await), TerminalMode::Work)).await.unwrap();

    assert!(seed.worktree_host.branches().is_empty());
}

#[tokio::test]
async fn a_session_is_not_opened_when_its_branch_cannot_be_made() {
    let (service, host, seed) = service().await;
    seed.keep_tasks_on_branches().await;
    seed.worktree_host.refuse("rekall/report");

    let refused = service.open(command(TerminalTarget::Task(seed.task("report").await), TerminalMode::Work)).await;

    assert!(refused.is_err());
    assert!(host.openings.lock().unwrap().is_empty());
}
