use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;
use rekall_core::{Ctx, EventBus};
use rekall_kernel::RekallError;
use rekall_store::{company, project, task, ProjectStatus};
use sea_orm::{ActiveModelTrait, IntoActiveModel};

use super::*;

/// Counts what the console asks of the worker; starting goes through the queue as the real one does.
#[derive(Default)]
struct FakeRunner {
    nudges: AtomicUsize,
    stops: AtomicUsize,
}

#[async_trait]
impl QueueRunner for FakeRunner {
    fn nudge(&self) {
        self.nudges.fetch_add(1, Ordering::SeqCst);
    }

    async fn start(&self, queue: &RunQueueService, start_at: Option<Instant>) -> Result<RunQueueView> {
        queue.start(start_at).await
    }

    async fn stop(&self) -> Result<RunQueueView> {
        self.stops.fetch_add(1, Ordering::SeqCst);
        Err(RekallError::conflict("The fake has nothing to stop."))
    }
}

struct World {
    service: RunQueueControlService,
    runner: Arc<FakeRunner>,
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
    let task = task::Model::new("report".into(), "Report".into(), project.id, now);
    task.clone().into_active_model().insert(&database.conn).await.unwrap();
    let runner = Arc::new(FakeRunner::default());
    World { service: RunQueueControlService::new(RunQueueService::new(ctx), runner.clone()), runner, task_id: task.id }
}

#[tokio::test]
async fn adding_a_task_wakes_the_worker() {
    let world = world().await;

    let queued = world.service.add(world.task_id).await.unwrap();

    assert_eq!(queued.items.len(), 1);
    assert_eq!(world.runner.nudges.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn a_refused_add_leaves_the_worker_asleep() {
    let world = world().await;

    assert!(world.service.add(Id::random()).await.is_err());

    assert_eq!(world.runner.nudges.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn stopping_is_the_workers_to_do() {
    let world = world().await;

    assert!(world.service.stop().await.is_err());

    assert_eq!(world.runner.stops.load(Ordering::SeqCst), 1);
}
