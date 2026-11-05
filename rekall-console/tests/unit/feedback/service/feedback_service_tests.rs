use rekall_core::{EventBus, Services};
use rekall_kernel::Instant;
use rekall_store::{company, project, task_step, wrapup, FeedbackState, ProjectStatus, WrapupAuthor};

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
        description: None, blueprint_markdown: None, repo_folder: None, auto_commit: false, company_id: company.id,
        created_at: now, updated_at: now,
    };
    project.clone().into_active_model().insert(&database.conn).await.unwrap();
    let mut task = task::Model::new("report".into(), "Report".into(), project.id, now);
    task.description = Some("Export to CSV.".into());
    task.clone().into_active_model().insert(&database.conn).await.unwrap();
    let services = Services::new(ctx.clone(), std::sync::Arc::new(rekall_core::check::ScriptedCheckRunner::new()));
    let service = FeedbackService::new(ctx.clone(), services.review.clone());
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

    async fn claim_the_review_line(&self) {
        self.set_review_state(TaskStepState::Claimed).await;
    }

    async fn set_review_state(&self, state: TaskStepState) {
        let mut task = self.task().await;
        task.mark_review_state(state);
        task.into_active_model().reset_all().update(&self.ctx.db).await.unwrap();
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

    async fn add(&self, kind: FeedbackKind, body: &str) -> FeedbackView {
        self.service.add(self.task_id, AddFeedbackCommand { kind, body: body.into() }).await.unwrap()
    }

    async fn reply_to(&self, seq: i32) {
        self.services.feedback.reply(Some("app"), "report", vec![(seq, "Done.".into())]).await.unwrap();
    }
}

#[tokio::test]
async fn feedback_is_added_pending_and_numbered_in_order() {
    let world = world().await;

    let first = world.add(FeedbackKind::Implementation, "  The totals are wrong  ").await;
    let second = world.add(FeedbackKind::Plan, "Split step 2").await;

    assert_eq!((first.seq, second.seq), (1, 2));
    assert_eq!((first.state, first.body_markdown.as_str()), (FeedbackState::Pending, "The totals are wrong"));
    assert_eq!(world.thread().await.len(), 2);
}

#[tokio::test]
async fn a_blank_body_and_a_typed_revision_are_refused_as_the_callers_mistake() {
    let world = world().await;

    let blank = world.service.add(world.task_id, AddFeedbackCommand { kind: FeedbackKind::Plan, body: "  ".into() }).await.unwrap_err();
    let revision = world.service.add(world.task_id, AddFeedbackCommand { kind: FeedbackKind::Revision, body: "x".into() }).await.unwrap_err();

    assert!(blank.is_illegal_argument() && revision.is_illegal_argument());
    assert!(world.thread().await.is_empty());
}

#[tokio::test]
async fn feedback_on_the_work_reopens_a_claimed_task_with_no_steps_and_feedback_on_the_plan_does_not() {
    let world = world().await;
    world.claim_the_review_line().await;

    world.add(FeedbackKind::Plan, "Split step 2").await;
    assert_eq!(world.task().await.review_state, TaskStepState::Claimed);

    world.add(FeedbackKind::Implementation, "The totals are wrong").await;
    assert_eq!(world.task().await.review_state, TaskStepState::Open);
}

#[tokio::test]
async fn feedback_on_the_work_of_a_task_with_steps_leaves_its_review_line_alone() {
    let world = world().await;
    world.add_a_step(TaskStepState::Claimed).await;
    world.claim_the_review_line().await;

    world.add(FeedbackKind::Implementation, "The totals are wrong").await;

    assert_eq!(world.task().await.review_state, TaskStepState::Claimed);
}

#[tokio::test]
async fn a_pending_entry_can_be_edited_and_deleted() {
    let world = world().await;
    let entry = world.add(FeedbackKind::Implementation, "first words").await;

    let edited = world.service.edit(entry.id, " better words ").await.unwrap();
    assert_eq!(edited.body_markdown, "better words");

    world.service.delete(entry.id).await.unwrap();
    assert!(world.thread().await.is_empty());
}

#[tokio::test]
async fn an_applied_entry_can_no_longer_be_edited_or_deleted() {
    let world = world().await;
    let entry = world.add(FeedbackKind::Implementation, "first words").await;
    world.reply_to(entry.seq).await;

    let edit = world.service.edit(entry.id, "new words").await.unwrap_err();
    let delete = world.service.delete(entry.id).await.unwrap_err();

    assert!(matches!(edit, RekallError::Conflict(_)) && matches!(delete, RekallError::Conflict(_)), "{edit:?} {delete:?}");
    assert_eq!(world.thread().await[0].body_markdown, "first words");
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
    assert_eq!(thread[0].kind, FeedbackKind::Revision);
    assert_eq!(thread[0].quoted_markdown.as_deref(), Some("Export to CSV."));
    assert_eq!(thread[0].body_markdown, "Export to CSV, XLSX and PDF.");
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
    world.edit_description("Export to CSV and XLSX.").await;
    let revision = world.thread().await.remove(0);

    world.service.discard_revision(revision.id).await.unwrap();

    assert_eq!(world.task().await.description.as_deref(), Some("Export to CSV."));
    assert!(world.thread().await.is_empty());
}

#[tokio::test]
async fn only_a_pending_revision_can_be_discarded_and_a_revision_cannot_be_edited_or_deleted() {
    let world = world().await;
    world.evaluate_with_a_wrapup().await;
    world.edit_description("Export to CSV and XLSX.").await;
    let revision = world.thread().await.remove(0);
    let correction = world.add(FeedbackKind::Implementation, "The totals are wrong").await;

    assert!(matches!(world.service.discard_revision(correction.id).await.unwrap_err(), RekallError::Conflict(_)));
    assert!(matches!(world.service.edit(revision.id, "x").await.unwrap_err(), RekallError::Conflict(_)));
    assert!(matches!(world.service.delete(revision.id).await.unwrap_err(), RekallError::Conflict(_)));
    world.reply_to(revision.seq).await;
    assert!(matches!(world.service.discard_revision(revision.id).await.unwrap_err(), RekallError::Conflict(_)));
    assert_eq!(world.task().await.description.as_deref(), Some("Export to CSV and XLSX."));
}
