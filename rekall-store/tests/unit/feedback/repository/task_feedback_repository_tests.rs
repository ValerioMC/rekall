use super::*;
use crate::enums::ProjectStatus;
use crate::{company, project, task};
use rekall_kernel::Instant;
use sea_orm::{ActiveModelTrait, IntoActiveModel};

async fn a_task(db: &impl ConnectionTrait) -> task::Model {
    let now = Instant::now();
    let company = company::Model { id: Id::random(), name: "Acme".into(), description: None, created_at: now, updated_at: now };
    company.clone().into_active_model().insert(db).await.unwrap();
    let project = project::Model {
        id: Id::random(), label: "vega".into(), title: "Vega".into(), status: ProjectStatus::Active, icon: "folder".into(),
        description: None, blueprint_markdown: None, repo_folder: None, auto_commit: false, company_id: company.id,
        created_at: now, updated_at: now,
    };
    project.clone().into_active_model().insert(db).await.unwrap();
    let task = task::Model::new("report".into(), "Report".into(), project.id, now);
    task.clone().into_active_model().insert(db).await.unwrap();
    task
}

async fn add(db: &impl ConnectionTrait, task: &task::Model, kind: FeedbackKind, body: &str) -> Model {
    let seq = next_seq(db, task.id).await.unwrap();
    let quoted = kind.is_revision().then(|| "before".to_string());
    let entry = Model::new(task.id, seq, kind, quoted, body.into(), None, Instant::now());
    entry.clone().into_active_model().insert(db).await.unwrap();
    entry
}

async fn apply(db: &impl ConnectionTrait, entry: &Model) {
    let mut applied = entry.clone();
    applied.mark_applied("Done.".into(), Instant::now());
    applied.into_active_model().reset_all().update(db).await.unwrap();
}

#[tokio::test]
async fn the_first_entry_of_a_task_is_number_one_and_each_next_one_follows() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let task = a_task(db).await;
    assert_eq!(next_seq(db, task.id).await.unwrap(), 1);

    add(db, &task, FeedbackKind::Implementation, "one").await;
    add(db, &task, FeedbackKind::Plan, "two").await;

    assert_eq!(next_seq(db, task.id).await.unwrap(), 3);
    let seqs: Vec<i32> = list_of_task(db, task.id).await.unwrap().iter().map(|entry| entry.seq).collect();
    assert_eq!(seqs, [1, 2]);
}

#[tokio::test]
async fn only_the_pending_entries_are_pending() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let task = a_task(db).await;
    let first = add(db, &task, FeedbackKind::Implementation, "one").await;
    add(db, &task, FeedbackKind::Plan, "two").await;
    apply(db, &first).await;

    let pending = pending_of_task(db, task.id).await.unwrap();

    assert_eq!(pending.iter().map(|entry| entry.seq).collect::<Vec<_>>(), [2]);
    assert_eq!(find_by_seq(db, task.id, 1).await.unwrap().unwrap().state, FeedbackState::Applied);
}

#[tokio::test]
async fn the_open_revision_is_the_pending_one_and_none_once_it_is_applied() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let task = a_task(db).await;
    assert!(latest_open_revision(db, task.id).await.unwrap().is_none());

    let revision = add(db, &task, FeedbackKind::Revision, "after").await;
    add(db, &task, FeedbackKind::Implementation, "a correction").await;
    assert_eq!(latest_open_revision(db, task.id).await.unwrap().unwrap().id, revision.id);

    apply(db, &revision).await;
    assert!(latest_open_revision(db, task.id).await.unwrap().is_none());
}

#[tokio::test]
async fn a_second_entry_with_the_same_number_is_refused_by_the_schema() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let task = a_task(db).await;
    add(db, &task, FeedbackKind::Implementation, "one").await;

    let again = Model::new(task.id, 1, FeedbackKind::Plan, None, "two".into(), None, Instant::now());

    assert!(again.into_active_model().insert(db).await.is_err());
}

#[tokio::test]
async fn deleting_the_task_takes_its_thread() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let task = a_task(db).await;
    add(db, &task, FeedbackKind::Implementation, "one").await;

    task::Entity::delete_by_id(task.id).exec(db).await.unwrap();

    assert!(list_of_task(db, task.id).await.unwrap().is_empty());
}

async fn take(db: &impl ConnectionTrait, entry: &Model, pass: i32) {
    let mut taken = entry.clone();
    taken.take(pass, Instant::now());
    taken.into_active_model().reset_all().update(db).await.unwrap();
}

#[tokio::test]
async fn running_and_open_entries_are_told_apart_from_pending_and_applied_ones() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let task = a_task(db).await;
    let applied = add(db, &task, FeedbackKind::Implementation, "one").await;
    let running = add(db, &task, FeedbackKind::Plan, "two").await;
    add(db, &task, FeedbackKind::Implementation, "three").await;
    apply(db, &applied).await;
    take(db, &running, 1).await;

    let running_seqs: Vec<i32> = running_of_task(db, task.id).await.unwrap().iter().map(|entry| entry.seq).collect();
    let open_seqs: Vec<i32> = open_of_task(db, task.id).await.unwrap().iter().map(|entry| entry.seq).collect();
    let pending_seqs: Vec<i32> = pending_of_task(db, task.id).await.unwrap().iter().map(|entry| entry.seq).collect();

    assert_eq!(running_seqs, [2]);
    assert_eq!(open_seqs, [2, 3]);
    assert_eq!(pending_seqs, [3]);
}

#[tokio::test]
async fn the_latest_pass_is_zero_until_an_entry_is_taken_and_then_the_highest_number() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let task = a_task(db).await;
    let first = add(db, &task, FeedbackKind::Implementation, "one").await;
    let second = add(db, &task, FeedbackKind::Plan, "two").await;
    assert_eq!(latest_pass(db, task.id).await.unwrap(), 0);

    take(db, &first, 1).await;
    take(db, &second, 2).await;

    assert_eq!(latest_pass(db, task.id).await.unwrap(), 2);
}

#[tokio::test]
async fn an_entry_from_before_passes_has_no_pass_and_a_follow_up_keeps_what_it_follows() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let task = a_task(db).await;
    let legacy = add(db, &task, FeedbackKind::Implementation, "old").await;
    apply(db, &legacy).await;
    let follow_up = Model::new(task.id, 2, FeedbackKind::Implementation, None, "still wrong".into(), Some(1), Instant::now());
    follow_up.into_active_model().insert(db).await.unwrap();

    let first = find_by_seq(db, task.id, 1).await.unwrap().unwrap();
    let second = find_by_seq(db, task.id, 2).await.unwrap().unwrap();

    assert_eq!((first.pass, first.taken_at, first.follows_seq), (None, None, None));
    assert_eq!(second.follows_seq, Some(1));
    assert_eq!(latest_pass(db, task.id).await.unwrap(), 0);
}
