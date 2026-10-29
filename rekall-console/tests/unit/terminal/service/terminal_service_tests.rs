use std::sync::Mutex;

use async_trait::async_trait;
use rekall_core::claude::TerminalMode;
use rekall_kernel::Instant;

use super::*;

/// Opens every terminal it is asked for and remembers each opening, so a test sees what reached it.
#[derive(Default)]
struct FakeHost {
    openings: Mutex<Vec<TerminalOpening>>,
    live: Mutex<Vec<TerminalView>>,
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
        task_label: None,
        task_title: None,
        skip_permissions: false,
        model: None,
        effort: None,
        live: true,
        started_at: now,
        last_activity_at: now,
        connections: Vec::new(),
    }
}

fn command(target: TerminalTarget, mode: TerminalMode) -> OpenTerminalCommand {
    OpenTerminalCommand { target, step_id: None, skip_permissions: true, model: Some("sonnet".into()), effort: None, mode }
}

fn service() -> (TerminalService, Arc<FakeHost>) {
    let host = Arc::new(FakeHost::default());
    (TerminalService::new(host.clone()), host)
}

#[tokio::test]
async fn a_terminal_on_a_task_is_opened_with_everything_the_command_asked_for() {
    let (service, host) = service();
    let task_id = Id::random();

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
    let (service, host) = service();

    let refused = service.open(command(TerminalTarget::Project(Id::random()), TerminalMode::Work)).await;
    let inventory = service.open(command(TerminalTarget::Project(Id::random()), TerminalMode::Inventory)).await;

    assert!(matches!(refused, Err(RekallError::IllegalArgument(_))));
    assert!(inventory.is_ok());
    assert_eq!(host.openings.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn an_unknown_terminal_is_not_found_on_read_and_on_close() {
    let (service, _) = service();

    assert!(matches!(service.get(Id::random()), Err(RekallError::NotFound(_))));
    assert!(matches!(service.close(Id::random()).await, Err(RekallError::NotFound(_))));
}

#[tokio::test]
async fn closing_a_live_terminal_takes_it_off_the_list() {
    let (service, _) = service();
    let opened = service.open(command(TerminalTarget::Task(Id::random()), TerminalMode::Work)).await.unwrap();

    service.close(opened.id).await.unwrap();

    assert!(service.list().is_empty());
}
