use rekall_core::{EventBus, Services};
use rekall_kernel::Instant;
use rekall_store::{company, project, task_step, wrapup, FeedbackState, ProjectStatus, TaskStepState, WrapupAuthor};

use super::*;

struct World {
    service: FeedbackService,
    services: Services,
    ctx: Ctx,
    task_id: Id,
}

async fn world() -> World {
    let database = rekall_store::open_in_memory().await.unwrap();
    let ctx = Ctx::new(database.conn.clone(), EventBus::new());
    let now = Instant::now();
    let company = company::Model { id: Id::random(), name: "Acme".into(), description: None, created_at: now, updated_at: now };
    company.clone().into_active_model().insert(&database.conn).await.unwrap();
    let project = project::Model {
        id: Id::random(), label: "app".into(), title: "App".into(), status: ProjectStatus::Active, icon: "folder".into(),
        description: None, blueprint_markdown: None, repo_folder: None, auto_commit: false, main_branch: None, pull_request_flow: false, task_key: String::new(), company_id: company.id,
        created_at: now, updated_at: now,
    };
    project.clone().into_active_model().insert(&database.conn).await.unwrap();
    let mut task = task::Model::new("report".into(), "Report".into(), project.id, now);
    task.description = Some("Export to CSV.".into());
    task.clone().into_active_model().insert(&database.conn).await.unwrap();
    let services = Services::new(ctx.clone(), std::sync::Arc::new(rekall_core::check::ScriptedCheckRunner::new()));
    let service = FeedbackService::new(ctx.clone());
    World { service, services, ctx, task_id: task.id }
}

impl World {
    async fn task(&self) -> task::Model {
        repo::task::require(&self.ctx.db, self.task_id).await.unwrap()
    }

    async fn evaluate_with_a_wrapup(&self) {
        let now = Instant::now();
        let wrapup = wrapup::Model { id: Id::random(), task_id: self.task_id, body_markdown: "built".into(), written_by: WrapupAuthor::Claude, created_at: now, updated_at: now };
        wrapup.into_active_model().insert(&self.ctx.db).await.unwrap();
    }

    async fn add_a_step(&self, state: TaskStepState) {
        let mut step = task_step::Model::new(self.task_id, "Build".into(), 0, Instant::now());
        step.state = state;
        step.into_active_model().insert(&self.ctx.db).await.unwrap();
    }

    async fn edit_description(&self, after: &str) {
        let before = self.task().await;
        self.save_and_track(before, after).await.unwrap();
    }

    async fn save_and_track(&self, before: task::Model, after: &str) -> Result<()> {
        in_write!(&self.ctx, |tx| {
            let mut task = before.clone();
            task.description = Some(after.to_string());
            task.clone().into_active_model().reset_all().update(tx.db()).await?;
            self.service.track_description_in(&mut tx, &task, before.description.as_deref(), task.description.as_deref()).await
        })
    }

    async fn thread(&self) -> Vec<FeedbackView> {
        self.service.list(self.task_id).await.unwrap()
    }

    async fn revise(&self, after: &str) -> FeedbackView {
        self.edit_description(after).await;
        self.thread().await.pop().unwrap()
    }

    async fn reply_to(&self, seq: i32) {
        self.services.feedback.reply(Some("app"), "report", vec![(seq, "Done.".into())]).await.unwrap();
    }
}

#[tokio::test]
async fn editing_the_description_of_a_task_nobody_evaluated_records_nothing() {
    let world = world().await;

    world.edit_description("Export to CSV and XLSX.").await;

    assert!(world.thread().await.is_empty());
}

#[tokio::test]
async fn editing_the_description_of_an_evaluated_task_opens_one_revision_that_later_edits_rewrite() {
    let world = world().await;
    world.evaluate_with_a_wrapup().await;

    world.edit_description("Export to CSV and XLSX.").await;
    world.edit_description("Export to CSV, XLSX and PDF.").await;

    let thread = world.thread().await;
    assert_eq!(thread.len(), 1);
    assert_eq!((thread[0].seq, thread[0].state), (1, FeedbackState::Pending));
    assert_eq!(thread[0].quoted_markdown.as_deref(), Some("Export to CSV."));
    assert_eq!(thread[0].body_markdown, "Export to CSV, XLSX and PDF.");
}

#[tokio::test]
async fn a_revision_is_applied_to_the_work_unless_the_task_has_steps() {
    let world = world().await;
    world.evaluate_with_a_wrapup().await;
    let on_the_work = world.revise("Export to CSV and XLSX.").await;
    world.reply_to(on_the_work.seq).await;
    world.add_a_step(TaskStepState::Open).await;

    let on_the_plan = world.revise("Export to CSV, XLSX and PDF.").await;

    assert_eq!((on_the_work.kind, on_the_plan.kind), (FeedbackKind::Implementation, FeedbackKind::Plan));
}

#[tokio::test]
async fn a_waiting_revision_can_be_pointed_at_the_plan_or_the_work_and_keeps_its_choice_when_rewritten() {
    let world = world().await;
    world.evaluate_with_a_wrapup().await;
    let revision = world.revise("Export to CSV and XLSX.").await;

    let retargeted = world.service.retarget(revision.id, FeedbackKind::Plan).await.unwrap();
    world.edit_description("Export to CSV, XLSX and PDF.").await;

    assert_eq!(retargeted.kind, FeedbackKind::Plan);
    assert_eq!(world.thread().await[0].kind, FeedbackKind::Plan);
}

#[tokio::test]
async fn undoing_the_edit_removes_the_revision() {
    let world = world().await;
    world.evaluate_with_a_wrapup().await;
    world.edit_description("Export to CSV and XLSX.").await;

    world.edit_description("Export to CSV.").await;

    assert!(world.thread().await.is_empty());
}

#[tokio::test]
async fn once_a_revision_is_applied_the_next_edit_opens_a_new_one_quoting_the_applied_text() {
    let world = world().await;
    world.evaluate_with_a_wrapup().await;
    world.edit_description("Export to CSV and XLSX.").await;
    world.reply_to(1).await;

    world.edit_description("Export to CSV, XLSX and PDF.").await;

    let thread = world.thread().await;
    assert_eq!(thread.len(), 2);
    assert_eq!((thread[1].seq, thread[1].state), (2, FeedbackState::Pending));
    assert_eq!(thread[1].quoted_markdown.as_deref(), Some("Export to CSV and XLSX."));
}

#[tokio::test]
async fn discarding_a_revision_puts_the_earlier_text_back_and_removes_it() {
    let world = world().await;
    world.evaluate_with_a_wrapup().await;
    let revision = world.revise("Export to CSV and XLSX.").await;

    world.service.discard_revision(revision.id).await.unwrap();

    assert_eq!(world.task().await.description.as_deref(), Some("Export to CSV."));
    assert!(world.thread().await.is_empty());
}

#[tokio::test]
async fn an_applied_revision_can_be_neither_discarded_nor_pointed_elsewhere() {
    let world = world().await;
    world.evaluate_with_a_wrapup().await;
    let revision = world.revise("Export to CSV and XLSX.").await;
    world.reply_to(revision.seq).await;

    let discard = world.service.discard_revision(revision.id).await.unwrap_err();
    let retarget = world.service.retarget(revision.id, FeedbackKind::Plan).await.unwrap_err();

    assert!(matches!(discard, RekallError::Conflict(_)) && matches!(retarget, RekallError::Conflict(_)), "{discard:?} {retarget:?}");
    assert_eq!(world.task().await.description.as_deref(), Some("Export to CSV and XLSX."));
}

#[tokio::test]
async fn a_pass_takes_the_waiting_revision_under_one_number() {
    let world = world().await;
    world.evaluate_with_a_wrapup().await;
    world.edit_description("Export to CSV and XLSX.").await;

    let taken = world.service.start_pass(world.task_id).await.unwrap();

    assert_eq!(taken.iter().map(|entry| entry.seq).collect::<Vec<_>>(), [1]);
    assert!(taken.iter().all(|entry| entry.state == FeedbackState::Running && entry.pass == Some(1) && entry.taken_at.is_some()));
}

#[tokio::test]
async fn a_revision_made_during_a_pass_waits_and_forms_the_next_one() {
    let world = world().await;
    world.evaluate_with_a_wrapup().await;
    world.edit_description("Export to CSV and XLSX.").await;
    world.service.start_pass(world.task_id).await.unwrap();
    let late = world.revise("Export to CSV, XLSX and PDF.").await;
    assert_eq!((late.seq, late.state, late.pass), (2, FeedbackState::Pending, None));
    world.reply_to(1).await;

    let taken = world.service.start_pass(world.task_id).await.unwrap();

    assert_eq!(taken.iter().map(|entry| (entry.seq, entry.pass)).collect::<Vec<_>>(), [(2, Some(2))]);
    assert_eq!(world.thread().await[0].pass, Some(1));
}

#[tokio::test]
async fn a_second_pass_is_refused_while_one_runs_and_so_is_a_pass_with_nothing_waiting() {
    let world = world().await;
    world.evaluate_with_a_wrapup().await;
    let nothing = world.service.start_pass(world.task_id).await.unwrap_err();
    world.edit_description("Export to CSV and XLSX.").await;
    world.service.start_pass(world.task_id).await.unwrap();
    world.edit_description("Export to CSV, XLSX and PDF.").await;

    let running = world.service.start_pass(world.task_id).await.unwrap_err();

    assert!(nothing.is_illegal_argument(), "{nothing:?}");
    assert!(matches!(running, RekallError::Conflict(_)) && running.message().contains("Pass 1 is still running"), "{running:?}");
    assert_eq!(world.thread().await[1].state, FeedbackState::Pending);
}

#[tokio::test]
async fn a_revision_in_a_pass_can_be_neither_discarded_nor_pointed_elsewhere() {
    let world = world().await;
    world.evaluate_with_a_wrapup().await;
    let revision = world.revise("Export to CSV and XLSX.").await;
    world.service.start_pass(world.task_id).await.unwrap();

    let discard = world.service.discard_revision(revision.id).await.unwrap_err();
    let retarget = world.service.retarget(revision.id, FeedbackKind::Plan).await.unwrap_err();

    assert!([&discard, &retarget].iter().all(|refused| matches!(refused, RekallError::Conflict(_)) && refused.message().contains("pass 1")), "{discard:?} {retarget:?}");
    assert_eq!(world.task().await.description.as_deref(), Some("Export to CSV and XLSX."));
}
