//! The services against a real database file. Where the Java tests mocked a repository or the
//! git wrapper, these arrange the same situation for real: the rows in the database, the
//! repository on disk, a hook that refuses the commit.

mod support;

use std::fs;

use chrono::TimeDelta;
use rekall_common::{Id, Instant};
use rekall_model::{task_revision, RevisionKind, TaskStepState, WrapupAuthor};
use rekall_service::commit::AutoCommitStatus;
use rekall_service::timeentry::IDLE_AFTER;
use rekall_service::revision::{RevisionTrigger, KEPT_PER_KIND};
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};
use support::{commit, git, init_with_identity, world, world_at};

// ------------------------------------------------------------------------------ auto-commit

async fn ready_repo() -> tempfile::TempDir {
    let repo = tempfile::tempdir().unwrap();
    init_with_identity(repo.path());
    fs::write(repo.path().join("seed.txt"), "seed").unwrap();
    git(repo.path(), &["add", "-A"]);
    commit(repo.path(), "seed");
    repo
}

#[tokio::test]
async fn a_project_that_does_not_auto_commit_does_nothing_and_never_touches_git() {
    let world = world().await;
    let repo = ready_repo().await;
    fs::write(repo.path().join("pending.txt"), "x").unwrap();
    let project = world.project("vega", Some(&repo.path().to_string_lossy()), false).await;
    let task = world.task(&project, "report-builder").await;
    let step = world.step(&task, "Wire the export endpoint", 2, TaskStepState::Claimed).await;

    let outcome = world.services.auto_commit.after_step_claim(task.id, step.id, None).await.unwrap();

    assert_eq!(outcome.status, AutoCommitStatus::Off);
    assert!(outcome.describe().is_empty());
    assert_eq!(git(repo.path(), &["log", "-1", "--format=%s"]), "seed");
}

#[tokio::test]
async fn a_step_claim_commits_the_pending_changes_and_logs_the_commit_against_that_step() {
    let world = world().await;
    let repo = ready_repo().await;
    fs::create_dir_all(repo.path().join("src")).unwrap();
    fs::write(repo.path().join("src/export.ts"), "export").unwrap();
    let project = world.project("vega", Some(&repo.path().to_string_lossy()), true).await;
    let task = world.task(&project, "report-builder").await;
    for (at, title) in ["One", "Two"].iter().enumerate() {
        world.step(&task, title, at as i32, TaskStepState::Open).await;
    }
    let step = world.step(&task, "Wire the export endpoint", 2, TaskStepState::Claimed).await;

    let outcome = world.services.auto_commit.after_step_claim(task.id, step.id, None).await.unwrap();

    assert_eq!(outcome.status, AutoCommitStatus::Committed);
    let reference = outcome.reference.clone().unwrap();
    assert_eq!(reference.step_id, Some(step.id));
    assert_eq!(reference.commit_hash, git(repo.path(), &["rev-parse", "HEAD"]));
    let message = git(repo.path(), &["log", "-1", "--format=%B"]);
    assert_eq!(message, "feat: Wire the export endpoint\n\nRefs: project:vega task:report-builder, step 3");
    assert!(outcome.describe().contains(&format!("`{}`", &reference.commit_hash[..7])));
    assert!(outcome.describe().contains("\"Wire the export endpoint\""));
}

#[tokio::test]
async fn the_claiming_sessions_own_message_becomes_the_commit_message() {
    let world = world().await;
    let repo = ready_repo().await;
    fs::write(repo.path().join("export.ts"), "export").unwrap();
    let project = world.project("vega", Some(&repo.path().to_string_lossy()), true).await;
    let task = world.task(&project, "report-builder").await;
    let mut step = world.step(&task, "Wire the export endpoint", 2, TaskStepState::Claimed).await;
    step.body_markdown = Some("Detail that must not appear.".into());
    step.clone().into_active_model().reset_all().update(world.db()).await.unwrap();

    world
        .services
        .auto_commit
        .after_step_claim(task.id, step.id, Some("fix: stream the export\n\nIt no longer buffers the file."))
        .await
        .unwrap();

    assert_eq!(
        git(repo.path(), &["log", "-1", "--format=%B"]),
        "fix: stream the export\n\nIt no longer buffers the file.\n\nRefs: project:vega task:report-builder, step 3"
    );
}

#[tokio::test]
async fn a_wrapup_on_a_stepless_task_commits_against_the_task_and_one_with_steps_commits_nothing() {
    let world = world().await;
    let repo = ready_repo().await;
    fs::write(repo.path().join("README.md"), "docs").unwrap();
    let project = world.project("vega", Some(&repo.path().to_string_lossy()), true).await;
    let stepless = world.task(&project, "report-builder").await;

    let outcome = world.services.auto_commit.after_wrapup(stepless.id, None).await.unwrap();
    assert_eq!(outcome.status, AutoCommitStatus::Committed);
    assert_eq!(outcome.reference.as_ref().unwrap().step_id, None);
    assert!(outcome.describe().contains("against the task"));
    assert_eq!(git(repo.path(), &["log", "-1", "--format=%s"]), "docs: Title of report-builder");

    let with_steps = world.task(&project, "checklist").await;
    world.step(&with_steps, "Open step", 0, TaskStepState::Open).await;
    fs::write(repo.path().join("more.txt"), "more").unwrap();
    assert_eq!(world.services.auto_commit.after_wrapup(with_steps.id, None).await.unwrap().status, AutoCommitStatus::Off);
}

#[tokio::test]
async fn a_clean_tree_is_skipped_and_nothing_is_logged() {
    let world = world().await;
    let repo = ready_repo().await;
    let project = world.project("vega", Some(&repo.path().to_string_lossy()), true).await;
    let task = world.task(&project, "report-builder").await;
    let step = world.step(&task, "Step", 0, TaskStepState::Claimed).await;

    let outcome = world.services.auto_commit.after_step_claim(task.id, step.id, None).await.unwrap();

    assert_eq!(outcome.status, AutoCommitStatus::Skipped);
    assert!(outcome.describe().contains("nothing to commit"));
    assert!(world.services.commit_references.find_all().await.unwrap().is_empty());
}

#[tokio::test]
async fn a_folder_that_is_not_a_repository_fails_the_commit_not_the_claim() {
    let world = world().await;
    let plain = tempfile::tempdir().unwrap();
    let project = world.project("vega", Some(&plain.path().to_string_lossy()), true).await;
    let task = world.task(&project, "report-builder").await;
    let step = world.step(&task, "Step", 0, TaskStepState::Claimed).await;

    let outcome = world.services.auto_commit.after_step_claim(task.id, step.id, None).await.unwrap();

    assert_eq!(outcome.status, AutoCommitStatus::Failed);
    assert!(outcome.describe().contains("not a git repository"));
    assert!(outcome.describe().contains("The claim stands"));
}

#[tokio::test]
async fn git_refusing_the_commit_is_reported_with_gits_reason() {
    let world = world().await;
    let repo = ready_repo().await;
    let hook = repo.path().join(".git/hooks/pre-commit");
    fs::write(&hook, "#!/bin/sh\necho 'hook rejected' >&2\nexit 1\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&hook, fs::Permissions::from_mode(0o755)).unwrap();
    }
    fs::write(repo.path().join("a.ts"), "a").unwrap();
    let project = world.project("vega", Some(&repo.path().to_string_lossy()), true).await;
    let task = world.task(&project, "report-builder").await;
    let step = world.step(&task, "Step", 0, TaskStepState::Claimed).await;

    let outcome = world.services.auto_commit.after_step_claim(task.id, step.id, None).await.unwrap();

    assert_eq!(outcome.status, AutoCommitStatus::Failed);
    assert!(outcome.detail.unwrap().contains("hook rejected"));
    assert!(world.services.commit_references.find_all().await.unwrap().is_empty());
}

#[tokio::test]
async fn an_unknown_task_or_step_is_nothing_to_commit_for() {
    let world = world().await;
    let project = world.project("vega", None, true).await;
    let task = world.task(&project, "report-builder").await;
    let step = world.step(&task, "Step", 0, TaskStepState::Claimed).await;
    let auto = &world.services.auto_commit;
    assert_eq!(auto.after_step_claim(Id::random(), step.id, None).await.unwrap().status, AutoCommitStatus::Off);
    assert_eq!(auto.after_step_claim(task.id, Id::random(), None).await.unwrap().status, AutoCommitStatus::Off);
    assert_eq!(auto.after_wrapup(Id::random(), None).await.unwrap().status, AutoCommitStatus::Off);
}

// ------------------------------------------------------------------------------ commit references

#[tokio::test]
async fn pressing_the_button_twice_on_the_same_commit_does_not_create_a_second_row_but_a_step_gets_its_own() {
    let world = world().await;
    let repo = ready_repo().await;
    let project = world.project("vega", Some(&repo.path().to_string_lossy()), false).await;
    let task = world.task(&project, "report-builder").await;
    let step = world.step(&task, "Step", 0, TaskStepState::Open).await;
    let references = &world.services.commit_references;

    let first = references.record_latest_commit(task.id, None).await.unwrap();
    let again = references.record_latest_commit(task.id, None).await.unwrap();
    assert_eq!(first.id, again.id);
    let on_step = references.record_latest_commit(task.id, Some(step.id)).await.unwrap();
    assert_ne!(on_step.id, first.id);
    assert_eq!(on_step.step_title.as_deref(), Some("Step"));
    assert_eq!(references.find_all().await.unwrap().len(), 2);
}

#[tokio::test]
async fn the_refusals_of_the_commit_log_name_their_reason() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "report-builder").await;
    let other = world.task(&project, "other").await;
    let foreign = world.step(&other, "Elsewhere", 0, TaskStepState::Open).await;
    let references = &world.services.commit_references;

    let no_folder = references.record_latest_commit(task.id, None).await.unwrap_err();
    assert!(no_folder.is_illegal_argument() && no_folder.message().contains("folder"));
    assert!(references.recent_commits(task.id).await.unwrap_err().message().contains("folder"));
    let unknown = references.record_latest_commit(Id::random(), None).await.unwrap_err();
    assert!(matches!(unknown, rekall_common::RekallError::NotFound(_)));
    let wrong_step = references.record_latest_commit(task.id, Some(foreign.id)).await.unwrap_err();
    assert!(wrong_step.message().contains("does not belong"));
    let missing_label = references.record_commit_by_anchor(Some("vega"), "missing", None, None).await.unwrap_err();
    assert!(matches!(missing_label, rekall_common::RekallError::UnknownAnchor(_)));
    assert!(matches!(references.diff_for(Id::random()).await.unwrap_err(), rekall_common::RekallError::NotFound(_)));
    assert!(matches!(references.delete(Id::random()).await.unwrap_err(), rekall_common::RekallError::NotFound(_)));
    assert!(matches!(references.set_in_context(Id::random(), Some(true)).await.unwrap_err(), rekall_common::RekallError::NotFound(_)));
}

#[tokio::test]
async fn a_logged_commit_is_chosen_for_the_context_and_a_missing_flag_reads_as_off() {
    let world = world().await;
    let repo = ready_repo().await;
    let project = world.project("vega", Some(&repo.path().to_string_lossy()), false).await;
    let task = world.task(&project, "report-builder").await;
    let references = &world.services.commit_references;
    let logged = references.record_commit_by_anchor(Some("vega"), "report-builder", None, None).await.unwrap();

    assert!(references.set_in_context(logged.id, Some(true)).await.unwrap().in_context);
    assert!(!references.set_in_context(logged.id, None).await.unwrap().in_context);
    assert!(references.diff_for(logged.id).await.unwrap().unwrap().contains("seed"));
    references.delete(logged.id).await.unwrap();
    assert!(references.find_all().await.unwrap().is_empty());
    let _ = task;
}

// ------------------------------------------------------------------------------ revisions

async fn revision(world: &support::World, task_id: Id, body: &str, created_at: Instant) {
    task_revision::Model {
        id: Id::random(),
        task_id,
        kind: RevisionKind::Wrapup,
        body_markdown: body.into(),
        written_by: Some(WrapupAuthor::Hand),
        written_at: None,
        created_at,
    }
    .into_active_model()
    .insert(world.db())
    .await
    .unwrap();
}

async fn keep(world: &support::World, task_id: Id, text: &str, author: Option<WrapupAuthor>, written_at: Option<Instant>, trigger: RevisionTrigger) -> bool {
    let mut tx = world.services.ctx.write().await.unwrap();
    let kept = world.services.revisions.keep_in(&mut tx, task_id, RevisionKind::Wrapup, Some(text), author, written_at, trigger).await;
    tx.finish(kept).await.unwrap()
}

#[tokio::test]
async fn the_text_a_sessions_write_replaces_is_kept_with_who_wrote_it_and_when() {
    let now = Instant::parse("2026-09-22T10:00:00Z").unwrap();
    let world = world_at(now).await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "t").await;
    let written_at = now.minus(TimeDelta::days(1));

    assert!(keep(&world, task.id, "old wrapup", Some(WrapupAuthor::Hand), Some(written_at), RevisionTrigger::ClaudeWrite).await);

    let kept = world.services.revisions.list(task.id, RevisionKind::Wrapup).await.unwrap();
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].body_markdown, "old wrapup");
    assert_eq!(kept[0].written_by, Some(WrapupAuthor::Hand));
    assert_eq!(kept[0].written_at, Some(written_at));
}

#[tokio::test]
async fn blank_text_or_text_the_newest_revision_already_holds_is_not_kept() {
    let now = Instant::parse("2026-09-22T10:00:00Z").unwrap();
    let world = world_at(now).await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "t").await;
    assert!(!keep(&world, task.id, "  ", None, None, RevisionTrigger::Deletion).await);
    revision(&world, task.id, "same text", now.minus(TimeDelta::days(3))).await;
    assert!(!keep(&world, task.id, "same text", None, None, RevisionTrigger::ClaudeWrite).await);
}

#[tokio::test]
async fn hand_edits_over_hand_written_text_keep_one_revision_per_ten_minute_window() {
    let now = Instant::parse("2026-09-22T10:00:00Z").unwrap();
    let world = world_at(now).await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "t").await;
    revision(&world, task.id, "before typing", now.minus(TimeDelta::minutes(3))).await;
    assert!(!keep(&world, task.id, "half typed", Some(WrapupAuthor::Hand), None, RevisionTrigger::HandEdit).await);

    let later = world.project("later", None, false).await;
    let other = world.task(&later, "t").await;
    revision(&world, other.id, "before typing", now.minus(TimeDelta::minutes(11))).await;
    assert!(keep(&world, other.id, "an hour later", Some(WrapupAuthor::Hand), None, RevisionTrigger::HandEdit).await);
}

#[tokio::test]
async fn a_hand_edit_over_a_sessions_text_and_a_deletion_or_restore_are_always_kept() {
    let now = Instant::parse("2026-09-22T10:00:00Z").unwrap();
    let world = world_at(now).await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "t").await;
    revision(&world, task.id, "older", now.minus(TimeDelta::minutes(1))).await;
    assert!(keep(&world, task.id, "claude's version", Some(WrapupAuthor::Claude), None, RevisionTrigger::HandEdit).await);
    assert!(keep(&world, task.id, "deleted", Some(WrapupAuthor::Hand), None, RevisionTrigger::Deletion).await);
    assert!(keep(&world, task.id, "restored over", Some(WrapupAuthor::Hand), None, RevisionTrigger::Restore).await);
}

#[tokio::test]
async fn past_thirty_revisions_of_one_text_the_oldest_are_dropped() {
    let now = Instant::parse("2026-09-22T10:00:00Z").unwrap();
    let world = world_at(now).await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "t").await;
    for at in 0..(KEPT_PER_KIND + 1) {
        revision(&world, task.id, &format!("revision {at}"), now.minus(TimeDelta::hours(100 - at as i64))).await;
    }
    keep(&world, task.id, "one more", None, None, RevisionTrigger::ClaudeWrite).await;
    let kept = world.services.revisions.list(task.id, RevisionKind::Wrapup).await.unwrap();
    assert_eq!(kept.len(), KEPT_PER_KIND);
    assert_eq!(kept[0].body_markdown, "one more");
    assert!(kept.iter().all(|r| r.body_markdown != "revision 0" && r.body_markdown != "revision 1"));
}

// ------------------------------------------------------------------------------ time entries

#[tokio::test]
async fn shutdown_stops_every_open_timer() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let first = world.task(&project, "first").await;
    let second = world.task(&project, "second").await;
    world.services.time_entries.start(first.id).await.unwrap();
    world.services.time_entries.start(second.id).await.unwrap();

    world.services.time_entries.stop_all_on_shutdown().await.unwrap();

    let entries = rekall_model::time_entry::Entity::find().all(world.db()).await.unwrap();
    assert_eq!(entries.len(), 2);
    assert!(entries.iter().all(|e| e.stopped_at.is_some()));
}

/// A world whose clock sits `hours` ahead of the rows the support helpers stamp, so those rows are idle.
async fn world_hours_ahead(hours: i64) -> support::World {
    world_at(Instant::now().plus(TimeDelta::hours(hours))).await
}

async fn open_entry_at(world: &support::World, task_id: Id, started_at: Instant) {
    let entry = rekall_model::time_entry::Model { id: Id::random(), task_id, started_at, stopped_at: None, created_at: started_at, updated_at: started_at };
    entry.into_active_model().insert(world.db()).await.unwrap();
}

#[tokio::test]
async fn an_idle_timer_is_stopped_at_the_tasks_last_write_and_a_busy_one_is_kept() {
    let world = world_hours_ahead(2).await;
    let project = world.project("vega", None, false).await;
    let idle = world.task(&project, "idle").await;
    let busy = world.task(&project, "busy").await;
    open_entry_at(&world, idle.id, idle.updated_at).await;
    world.services.time_entries.start(busy.id).await.unwrap();

    let stopped = world.services.time_entries.stop_idle(IDLE_AFTER).await.unwrap();

    assert_eq!(stopped, 1);
    let entries = rekall_model::time_entry::Entity::find().all(world.db()).await.unwrap();
    let idle_entry = entries.iter().find(|e| e.task_id == idle.id).unwrap();
    assert_eq!(idle_entry.stopped_at, Some(idle.updated_at));
    assert!(entries.iter().find(|e| e.task_id == busy.id).unwrap().stopped_at.is_none());
}

#[tokio::test]
async fn a_task_with_a_running_step_is_never_idle() {
    let world = world_hours_ahead(2).await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "working").await;
    world.step(&task, "Being built", 1, TaskStepState::Running).await;
    open_entry_at(&world, task.id, task.updated_at).await;

    let stopped = world.services.time_entries.stop_idle(IDLE_AFTER).await.unwrap();

    assert_eq!(stopped, 0);
    assert_eq!(running_entries(&world).await, 1);
}

async fn running_entries(world: &support::World) -> usize {
    let entries = rekall_model::time_entry::Entity::find().all(world.db()).await.unwrap();
    entries.iter().filter(|e| e.stopped_at.is_none()).count()
}

#[tokio::test]
async fn accepting_a_claimed_step_stops_the_tasks_timer() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "report").await;
    let step = world.step(&task, "export", 0, TaskStepState::Claimed).await;
    world.services.time_entries.start(task.id).await.unwrap();

    world.services.steps.edit(step.id, None, None, Some(true), None).await.unwrap();

    assert_eq!(running_entries(&world).await, 0);
}

#[tokio::test]
async fn ticking_an_open_step_by_hand_keeps_the_timer_running() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "report").await;
    let step = world.step(&task, "export", 0, TaskStepState::Open).await;
    world.services.time_entries.start(task.id).await.unwrap();

    world.services.steps.edit(step.id, None, None, Some(true), None).await.unwrap();

    assert_eq!(running_entries(&world).await, 1);
}

#[tokio::test]
async fn accepting_a_claimed_stepless_task_stops_its_timer_and_accepting_an_unclaimed_one_does_not() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let claimed = world.task(&project, "claimed").await;
    let open = world.task(&project, "open").await;
    let mut row = claimed.clone();
    row.mark_review_state(TaskStepState::Claimed);
    row.into_active_model().reset_all().update(world.db()).await.unwrap();
    world.services.time_entries.start(claimed.id).await.unwrap();
    world.services.time_entries.start(open.id).await.unwrap();

    let mut tx = rekall_service::Tx::write(&world.services.ctx).await.unwrap();
    world.services.review.accept_in(&mut tx, claimed.id).await.unwrap();
    world.services.review.accept_in(&mut tx, open.id).await.unwrap();
    tx.commit().await.unwrap();

    let entries = rekall_model::time_entry::Entity::find().all(world.db()).await.unwrap();
    let stopped = |task_id: Id| entries.iter().find(|e| e.task_id == task_id).unwrap().stopped_at.is_some();
    assert!(stopped(claimed.id));
    assert!(!stopped(open.id));
}

// ------------------------------------------------------------------------------ remaining Java cases

#[tokio::test]
async fn a_repository_with_no_user_email_is_refused_before_anything_is_staged() {
    let world = world().await;
    let repo = ready_repo().await;
    // A blank local value hides any global one, which is what "no email" looks like here.
    git(repo.path(), &["config", "user.email", ""]);
    fs::write(repo.path().join("pending.txt"), "x").unwrap();
    let project = world.project("vega", Some(&repo.path().to_string_lossy()), true).await;
    let task = world.task(&project, "report-builder").await;
    let step = world.step(&task, "Step", 0, TaskStepState::Claimed).await;

    let outcome = world.services.auto_commit.after_step_claim(task.id, step.id, None).await.unwrap();

    assert_eq!(outcome.status, AutoCommitStatus::Failed);
    assert!(outcome.describe().contains("user.email"), "{}", outcome.describe());
    assert_eq!(git(repo.path(), &["status", "--porcelain"]), "?? pending.txt", "nothing was staged");
}

#[tokio::test]
async fn a_commit_named_by_its_hash_is_read_by_that_hash_and_a_blank_hash_means_the_tip() {
    let world = world().await;
    let repo = ready_repo().await;
    fs::write(repo.path().join("later.txt"), "later").unwrap();
    git(repo.path(), &["add", "-A"]);
    commit(repo.path(), "The later one");
    let earlier = git(repo.path(), &["rev-parse", "HEAD~1"]);
    let tip = git(repo.path(), &["rev-parse", "HEAD"]);
    let project = world.project("vega", Some(&repo.path().to_string_lossy()), false).await;
    let task = world.task(&project, "report-builder").await;
    let references = &world.services.commit_references;

    let named = references.record_commit(task.id, None, Some(&format!(" {} ", &earlier[..7]))).await.unwrap();
    assert_eq!(named.commit_hash, earlier);
    assert_eq!(named.comment, "seed");

    let blank = references.record_commit(task.id, None, Some("   ")).await.unwrap();
    assert_eq!(blank.commit_hash, tip, "a blank hash is the plain button: the tip");
    assert_eq!(blank.comment, "The later one");

    let by_anchor = references.record_commit_by_anchor(Some("vega"), "report-builder", None, Some(&earlier)).await.unwrap();
    assert_eq!(by_anchor.id, named.id, "the MCP path passes the hash through, to the same row");
}

#[tokio::test]
async fn the_recent_log_is_read_from_the_tasks_project_folder_newest_first() {
    let world = world().await;
    let repo = ready_repo().await;
    fs::write(repo.path().join("later.txt"), "later").unwrap();
    git(repo.path(), &["add", "-A"]);
    commit(repo.path(), "The later one");
    let project = world.project("vega", Some(&repo.path().to_string_lossy()), false).await;
    let task = world.task(&project, "report-builder").await;

    let recent = world.services.commit_references.recent_commits(task.id).await.unwrap();

    let subjects: Vec<&str> = recent.iter().map(|entry| entry.subject.as_str()).collect();
    assert_eq!(subjects, ["The later one", "seed"]);
    assert_eq!(recent[0].hash, git(repo.path(), &["rev-parse", "HEAD"]));
}

#[tokio::test]
async fn under_three_characters_there_is_no_search() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    world.task(&project, "ab").await;
    assert!(world.services.search.search(Some("ab")).await.unwrap().is_empty());
    assert!(world.services.search.search(Some("  ")).await.unwrap().is_empty());
    assert!(world.services.search.search(None).await.unwrap().is_empty());
}

// ------------------------------------------------------------------------------ send back

#[tokio::test]
async fn a_step_sent_back_hands_the_next_session_its_earlier_pass_and_the_feedback_apart() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "report-builder").await;
    let step = world.step(&task, "Aggregate the rows", 0, TaskStepState::Open).await;
    let steps = &world.services.steps;
    steps.edit(step.id, None, Some("Sum by month"), None, None).await.unwrap();
    steps.transition(Some("vega"), "report-builder", Some("1"), TaskStepState::Claimed).await.unwrap();

    let sent_back = steps.send_back(step.id).await.unwrap();
    assert_eq!(sent_back.state, TaskStepState::Open);
    assert_eq!(sent_back.body_markdown, None);
    assert_eq!(sent_back.passes[0].detail_markdown.as_deref(), Some("Sum by month"));

    steps.edit(step.id, None, Some("Group by week instead"), None, None).await.unwrap();
    let record = world.services.context.load_task("vega", "report-builder").await.unwrap();
    let rendered = world.services.renderer.render_steps(&record.steps, None);

    assert!(rendered.contains("Previously you have executed this:"), "{rendered}");
    assert!(rendered.contains("<previous-pass number=\"1\""), "{rendered}");
    assert!(rendered.contains("Sum by month"), "{rendered}");
    let feedback_at = rendered.find("<feedback>").unwrap();
    assert!(rendered[feedback_at..].contains("Group by week instead"), "{rendered}");
}

#[tokio::test]
async fn only_a_claimed_step_can_be_sent_back() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "report-builder").await;
    let step = world.step(&task, "Aggregate the rows", 0, TaskStepState::Open).await;

    assert!(world.services.steps.send_back(step.id).await.is_err());
}
