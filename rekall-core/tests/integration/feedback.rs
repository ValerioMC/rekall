//! The thread a session answers: replies applied together or not at all, and what the context shows after.

mod support;

use rekall_core::feedback::FeedbackStreamEvent;
use rekall_kernel::Instant;
use rekall_store::repository as repo;
use rekall_store::{task, task_feedback, FeedbackKind, FeedbackState};
use sea_orm::{ActiveModelTrait, IntoActiveModel};
use support::{world, World};

async fn pending(world: &World, task: &task::Model, kind: FeedbackKind, body: &str) -> task_feedback::Model {
    let seq = repo::task_feedback::next_seq(world.db(), task.id).await.unwrap();
    let quoted = kind.is_revision().then(|| "the text before".to_string());
    let entry = task_feedback::Model::new(task.id, seq, kind, quoted, body.into(), None, Instant::now());
    entry.clone().into_active_model().insert(world.db()).await.unwrap();
    entry
}

async fn state_of(world: &World, task: &task::Model, seq: i32) -> FeedbackState {
    repo::task_feedback::find_by_seq(world.db(), task.id, seq).await.unwrap().unwrap().state
}

#[tokio::test]
async fn replying_applies_each_named_entry_and_keeps_the_others_pending() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "checkout").await;
    pending(&world, &task, FeedbackKind::Implementation, "The totals are wrong").await;
    pending(&world, &task, FeedbackKind::Plan, "Split step 2").await;
    pending(&world, &task, FeedbackKind::Implementation, "Rename the button").await;

    let replied = world
        .services
        .feedback
        .reply(Some("vega"), "checkout", vec![(1, "  Fixed the totals.  ".into()), (3, "Renamed it.".into())])
        .await
        .unwrap();

    assert_eq!((replied.applied, replied.still_open), (vec![1, 3], vec![2]));
    let first = repo::task_feedback::find_by_seq(world.db(), task.id, 1).await.unwrap().unwrap();
    assert_eq!((first.state, first.reply_markdown.as_deref()), (FeedbackState::Applied, Some("Fixed the totals.")));
    assert!(first.applied_at.is_some());
    assert_eq!(state_of(&world, &task, 2).await, FeedbackState::Pending);
}

#[tokio::test]
async fn replying_to_a_revision_changes_nothing_but_the_entry() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "checkout").await;
    pending(&world, &task, FeedbackKind::Revision, "the text now").await;

    world.services.feedback.reply(Some("vega"), "checkout", vec![(1, "Built it.".into())]).await.unwrap();

    assert_eq!(state_of(&world, &task, 1).await, FeedbackState::Applied);
    let reread = repo::task::require(world.db(), task.id).await.unwrap();
    assert_eq!(reread.description, task.description);
}

#[tokio::test]
async fn an_unknown_seq_an_applied_entry_a_blank_reply_and_a_long_reply_are_refused_with_nothing_written() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "checkout").await;
    pending(&world, &task, FeedbackKind::Implementation, "one").await;
    pending(&world, &task, FeedbackKind::Implementation, "two").await;
    world.services.feedback.reply(Some("vega"), "checkout", vec![(2, "Done.".into())]).await.unwrap();

    let unknown = world.services.feedback.reply(Some("vega"), "checkout", vec![(1, "Ok.".into()), (9, "Ok.".into())]).await.unwrap_err();
    let applied = world.services.feedback.reply(Some("vega"), "checkout", vec![(1, "Ok.".into()), (2, "Again.".into())]).await.unwrap_err();
    let blank = world.services.feedback.reply(Some("vega"), "checkout", vec![(1, "   ".into())]).await.unwrap_err();
    let long = world.services.feedback.reply(Some("vega"), "checkout", vec![(1, "x".repeat(2_001))]).await.unwrap_err();
    let empty = world.services.feedback.reply(Some("vega"), "checkout", Vec::new()).await.unwrap_err();

    assert!(unknown.message().contains("no entry with seq 9"), "{unknown:?}");
    assert!(applied.message().contains("Entry 2 is already applied"), "{applied:?}");
    assert!(blank.message().contains("reply to entry 1 is blank"), "{blank:?}");
    assert!(long.message().contains("capped at 2000 characters and this one is 2001"), "{long:?}");
    assert!(empty.is_illegal_argument());
    assert_eq!(state_of(&world, &task, 1).await, FeedbackState::Pending, "a refused call writes nothing, even the entries before the bad one");
}

#[tokio::test]
async fn a_reply_announces_the_thread_after_it_commits() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "checkout").await;
    pending(&world, &task, FeedbackKind::Implementation, "one").await;
    let mut events = world.services.ctx.events.subscribe();

    world.services.feedback.reply(Some("vega"), "checkout", vec![(1, "Done.".into())]).await.unwrap();

    let announced = events.recv().await.unwrap();
    assert_eq!(announced.downcast::<FeedbackStreamEvent>(), Some(&FeedbackStreamEvent { task_id: task.id }));
}

#[tokio::test]
async fn the_context_shows_pending_entries_in_full_and_applied_ones_as_one_line() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "checkout").await;
    pending(&world, &task, FeedbackKind::Implementation, "The totals are wrong, in detail").await;
    pending(&world, &task, FeedbackKind::Plan, "Split step 2").await;
    world.services.feedback.reply(Some("vega"), "checkout", vec![(1, "Fixed the totals.".into())]).await.unwrap();

    let rendered = world.services.renderer.render(&world.services.context.for_task(task.id).await.unwrap());

    assert!(rendered.contains("<entry seq=\"1\" kind=\"IMPLEMENTATION\" state=\"APPLIED\">Fixed the totals.</entry>"), "{rendered}");
    assert!(rendered.contains("<entry seq=\"2\" kind=\"PLAN\" state=\"PENDING\">\nSplit step 2\n</entry>"), "{rendered}");
    assert!(!rendered.contains("in detail"), "{rendered}");
}

async fn take(world: &World, task: &task::Model, seq: i32, pass: i32) {
    let before = repo::task_feedback::find_by_seq(world.db(), task.id, seq).await.unwrap().unwrap();
    let mut taken = before.clone();
    taken.take(pass, Instant::now());
    repo::update_if_changed(world.db(), &before, taken, Instant::now()).await.unwrap();
}

#[tokio::test]
async fn replying_answers_an_entry_in_a_pass_and_lists_what_is_still_open() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "checkout").await;
    pending(&world, &task, FeedbackKind::Implementation, "one").await;
    pending(&world, &task, FeedbackKind::Plan, "two").await;
    pending(&world, &task, FeedbackKind::Implementation, "three").await;
    take(&world, &task, 1, 1).await;
    take(&world, &task, 2, 1).await;

    let replied = world.services.feedback.reply(Some("vega"), "checkout", vec![(1, "Fixed.".into())]).await.unwrap();

    assert_eq!((replied.applied, replied.still_open), (vec![1], vec![2, 3]));
    let first = repo::task_feedback::find_by_seq(world.db(), task.id, 1).await.unwrap().unwrap();
    assert_eq!((first.state, first.pass), (FeedbackState::Applied, Some(1)));
}

#[tokio::test]
async fn releasing_gives_back_the_unanswered_entries_of_a_pass_and_leaves_the_rest() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "checkout").await;
    pending(&world, &task, FeedbackKind::Implementation, "one").await;
    pending(&world, &task, FeedbackKind::Plan, "two").await;
    pending(&world, &task, FeedbackKind::Implementation, "three").await;
    take(&world, &task, 1, 1).await;
    take(&world, &task, 2, 1).await;
    world.services.feedback.reply(Some("vega"), "checkout", vec![(1, "Fixed.".into())]).await.unwrap();
    let mut events = world.services.ctx.events.subscribe();

    world.services.feedback.release_running(task.id).await.unwrap();

    let released = repo::task_feedback::find_by_seq(world.db(), task.id, 2).await.unwrap().unwrap();
    assert_eq!((released.state, released.pass, released.taken_at), (FeedbackState::Pending, None, None));
    assert_eq!(state_of(&world, &task, 1).await, FeedbackState::Applied);
    assert_eq!(state_of(&world, &task, 3).await, FeedbackState::Pending);
    assert_eq!(events.recv().await.unwrap().downcast::<FeedbackStreamEvent>(), Some(&FeedbackStreamEvent { task_id: task.id }));
}

#[tokio::test]
async fn releasing_a_task_with_no_running_entry_announces_nothing() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "checkout").await;
    pending(&world, &task, FeedbackKind::Implementation, "one").await;
    let mut events = world.services.ctx.events.subscribe();

    world.services.feedback.release_running(task.id).await.unwrap();

    assert!(events.try_recv().is_err());
    assert_eq!(state_of(&world, &task, 1).await, FeedbackState::Pending);
}

#[tokio::test]
async fn the_context_shows_running_entries_in_full_with_their_pass_and_the_counts() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "checkout").await;
    pending(&world, &task, FeedbackKind::Implementation, "The totals are wrong, in detail").await;
    take(&world, &task, 1, 2).await;

    let rendered = world.services.renderer.render(&world.services.context.for_task(task.id).await.unwrap());

    assert!(rendered.contains("<thread pending=\"0\" running=\"1\" applied=\"0\">"), "{rendered}");
    assert!(rendered.contains("state=\"RUNNING\" pass=\"2\">\nThe totals are wrong, in detail\n</entry>"), "{rendered}");
}
