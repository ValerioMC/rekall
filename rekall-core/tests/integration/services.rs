//! The services against a real database file. Where the Java tests mocked a repository or the
//! git wrapper, these arrange the same situation for real: the rows in the database, the
//! repository on disk, a hook that refuses the commit.

mod support;

use std::fs;

use chrono::TimeDelta;
use rekall_kernel::{Id, Instant};
use rekall_store::TaskStepState;
use rekall_core::commit::AutoCommitStatus;
use rekall_core::component::ComponentReferences;
use rekall_core::step::{NewStep, Staging, StepEdit};
use rekall_core::timeentry::IDLE_AFTER;
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};
use support::{commit, git, init_with_identity, world, world_at};

// ------------------------------------------------------------------------------ step snapshot

async fn snapshot_files(world: &support::World, task_id: Id, step_id: Option<Id>) -> Option<Vec<(String, Option<String>)>> {
    let snapshot = rekall_store::repository::step_snapshot::find(world.db(), task_id, step_id).await.unwrap()?;
    let files = rekall_store::repository::step_snapshot::files_of(world.db(), snapshot.id).await.unwrap();
    Some(files.into_iter().map(|file| (file.path, file.content_hash)).collect())
}

#[tokio::test]
async fn a_snapshot_keeps_each_modified_new_and_deleted_file_with_its_content_hash() {
    let world = world().await;
    let repo = ready_repo().await;
    fs::write(repo.path().join("seed.txt"), "edited before the step").unwrap();
    fs::write(repo.path().join("notes.txt"), "not tracked").unwrap();
    fs::write(repo.path().join("gone.txt"), "to delete").unwrap();
    git(repo.path(), &["add", "gone.txt"]);
    commit(repo.path(), "add gone");
    fs::remove_file(repo.path().join("gone.txt")).unwrap();
    let project = world.project("vega", Some(&repo.path().to_string_lossy()), true).await;
    let task = world.task(&project, "report-builder").await;
    let step = world.step(&task, "Wire the export endpoint", 0, TaskStepState::Running).await;

    world.services.step_snapshots.take(task.id, Some(step.id)).await;

    let files = snapshot_files(&world, task.id, Some(step.id)).await.unwrap();
    let seed_hash = git(repo.path(), &["hash-object", "seed.txt"]);
    let notes_hash = git(repo.path(), &["hash-object", "notes.txt"]);
    assert_eq!(
        files,
        vec![
            ("gone.txt".to_string(), None),
            ("notes.txt".to_string(), Some(notes_hash)),
            ("seed.txt".to_string(), Some(seed_hash)),
        ]
    );
}

#[tokio::test]
async fn a_clean_tree_is_a_snapshot_with_no_files_not_a_missing_snapshot() {
    let world = world().await;
    let repo = ready_repo().await;
    let project = world.project("vega", Some(&repo.path().to_string_lossy()), true).await;
    let task = world.task(&project, "report-builder").await;
    let step = world.step(&task, "First", 0, TaskStepState::Running).await;

    world.services.step_snapshots.take(task.id, Some(step.id)).await;

    assert_eq!(snapshot_files(&world, task.id, Some(step.id)).await, Some(Vec::new()));
}

#[tokio::test]
async fn starting_a_step_again_replaces_its_snapshot_with_the_new_one() {
    let world = world().await;
    let repo = ready_repo().await;
    fs::write(repo.path().join("a.txt"), "first").unwrap();
    let project = world.project("vega", Some(&repo.path().to_string_lossy()), true).await;
    let task = world.task(&project, "report-builder").await;
    let step = world.step(&task, "First", 0, TaskStepState::Running).await;
    world.services.step_snapshots.take(task.id, Some(step.id)).await;

    fs::write(repo.path().join("a.txt"), "second").unwrap();
    fs::write(repo.path().join("b.txt"), "new").unwrap();
    world.services.step_snapshots.take(task.id, Some(step.id)).await;

    let files = snapshot_files(&world, task.id, Some(step.id)).await.unwrap();
    assert_eq!(files.iter().map(|(path, _)| path.as_str()).collect::<Vec<_>>(), vec!["a.txt", "b.txt"]);
    assert_eq!(files[0].1.as_deref(), Some(git(repo.path(), &["hash-object", "a.txt"]).as_str()));
    let kept = rekall_store::step_snapshot::Entity::find().all(world.db()).await.unwrap();
    assert_eq!(kept.len(), 1);
}

#[tokio::test]
async fn a_task_with_no_steps_keeps_its_snapshot_on_the_task_and_each_step_on_its_own() {
    let world = world().await;
    let repo = ready_repo().await;
    fs::write(repo.path().join("a.txt"), "x").unwrap();
    let project = world.project("vega", Some(&repo.path().to_string_lossy()), true).await;
    let stepless = world.task(&project, "typo").await;
    let with_steps = world.task(&project, "report-builder").await;
    let step = world.step(&with_steps, "First", 0, TaskStepState::Running).await;

    world.services.step_snapshots.take(stepless.id, None).await;
    world.services.step_snapshots.take(with_steps.id, Some(step.id)).await;

    assert_eq!(snapshot_files(&world, stepless.id, None).await.unwrap().len(), 1);
    assert_eq!(snapshot_files(&world, with_steps.id, Some(step.id)).await.unwrap().len(), 1);
    assert_eq!(snapshot_files(&world, with_steps.id, None).await, None);
}

#[tokio::test]
async fn a_project_that_does_not_auto_commit_keeps_no_snapshot() {
    let world = world().await;
    let repo = ready_repo().await;
    fs::write(repo.path().join("a.txt"), "x").unwrap();
    let project = world.project("vega", Some(&repo.path().to_string_lossy()), false).await;
    let task = world.task(&project, "report-builder").await;
    let step = world.step(&task, "First", 0, TaskStepState::Running).await;

    world.services.step_snapshots.take(task.id, Some(step.id)).await;

    assert_eq!(snapshot_files(&world, task.id, Some(step.id)).await, None);
}

#[tokio::test]
async fn a_folder_that_is_not_a_repository_keeps_no_snapshot_and_does_not_stop_the_work() {
    let world = world().await;
    let folder = tempfile::tempdir().unwrap();
    let project = world.project("vega", Some(&folder.path().to_string_lossy()), true).await;
    let task = world.task(&project, "report-builder").await;
    let step = world.step(&task, "First", 0, TaskStepState::Running).await;

    world.services.step_snapshots.take(task.id, Some(step.id)).await;

    assert_eq!(snapshot_files(&world, task.id, Some(step.id)).await, None);
}

#[tokio::test]
async fn a_project_folder_inside_the_repository_is_hashed_from_the_repository_root() {
    let world = world().await;
    let repo = ready_repo().await;
    fs::create_dir_all(repo.path().join("app")).unwrap();
    fs::write(repo.path().join("app/inner.txt"), "inner").unwrap();
    let project = world.project("vega", Some(&repo.path().join("app").to_string_lossy()), true).await;
    let task = world.task(&project, "report-builder").await;
    let step = world.step(&task, "First", 0, TaskStepState::Running).await;

    world.services.step_snapshots.take(task.id, Some(step.id)).await;

    let files = snapshot_files(&world, task.id, Some(step.id)).await.unwrap();
    assert_eq!(files, vec![("app/inner.txt".to_string(), Some(git(repo.path(), &["hash-object", "app/inner.txt"])))]);
}

#[tokio::test]
async fn deleting_a_step_removes_its_snapshot() {
    let world = world().await;
    let repo = ready_repo().await;
    fs::write(repo.path().join("a.txt"), "x").unwrap();
    let project = world.project("vega", Some(&repo.path().to_string_lossy()), true).await;
    let task = world.task(&project, "report-builder").await;
    let step = world.step(&task, "First", 0, TaskStepState::Open).await;
    world.services.step_snapshots.take(task.id, Some(step.id)).await;

    rekall_store::task_step::Entity::delete_by_id(step.id).exec(world.db()).await.unwrap();

    assert!(rekall_store::step_snapshot::Entity::find().all(world.db()).await.unwrap().is_empty());
    assert!(rekall_store::step_snapshot_file::Entity::find().all(world.db()).await.unwrap().is_empty());
}

#[tokio::test]
async fn a_step_is_started_once_however_many_times_running_is_asked_for() {
    let world = world().await;
    let project = world.project("vega", None, true).await;
    let task = world.task(&project, "report-builder").await;
    let step = world.step(&task, "First", 0, TaskStepState::Open).await;

    let first = world.services.steps.mark_running(Some(step.id)).await.unwrap();
    let again = world.services.steps.mark_running(Some(step.id)).await.unwrap();
    let from_the_tool = world
        .services
        .flow
        .transition(Some("vega"), "report-builder", Some("1"), TaskStepState::Running, None, None)
        .await
        .unwrap();

    assert!(first);
    assert!(!again);
    assert!(!from_the_tool.started);
}

#[tokio::test]
async fn asking_for_running_on_an_open_step_is_a_start_and_a_claim_is_not() {
    let world = world().await;
    let project = world.project("vega", None, true).await;
    let task = world.task(&project, "report-builder").await;
    world.step(&task, "First", 0, TaskStepState::Open).await;
    let flow = &world.services.flow;

    let started = flow.transition(Some("vega"), "report-builder", Some("1"), TaskStepState::Running, None, None).await.unwrap();
    let claimed = flow.transition(Some("vega"), "report-builder", Some("1"), TaskStepState::Claimed, None, None).await.unwrap();

    assert!(started.started);
    assert!(!claimed.started);
}

// ------------------------------------------------------------------------------ commit only the session's files

#[tokio::test]
async fn a_file_modified_before_the_step_started_stays_out_of_the_claim_commit_and_is_named() {
    let world = world().await;
    let repo = ready_repo().await;
    fs::write(repo.path().join("notes.txt"), "mine, unfinished").unwrap();
    fs::write(repo.path().join(".env.local"), "SECRET=1").unwrap();
    let project = world.project("vega", Some(&repo.path().to_string_lossy()), true).await;
    let task = world.task(&project, "report-builder").await;
    let step = world.step(&task, "Wire the export endpoint", 0, TaskStepState::Running).await;
    world.services.step_snapshots.take(task.id, Some(step.id)).await;
    fs::write(repo.path().join("export.ts"), "export").unwrap();

    let outcome = world.services.auto_commit.after_step_claim(task.id, step.id, None).await.unwrap();

    assert_eq!(outcome.status, AutoCommitStatus::Committed);
    assert_eq!(git(repo.path(), &["show", "--name-only", "--format=", "HEAD"]), "export.ts");
    assert_eq!(outcome.left_out, vec![".env.local".to_string(), "notes.txt".to_string()]);
    assert!(outcome.describe().contains("Left out because they were already modified before this work started: .env.local, notes.txt."));
    assert!(!outcome.describe().contains("No snapshot"));
    let left = git(repo.path(), &["status", "--porcelain"]);
    assert!(left.contains("?? notes.txt") && left.contains("?? .env.local"), "{left}");
}

#[tokio::test]
async fn a_file_modified_before_the_start_and_edited_again_by_the_session_goes_in() {
    let world = world().await;
    let repo = ready_repo().await;
    fs::write(repo.path().join("seed.txt"), "edited before").unwrap();
    let project = world.project("vega", Some(&repo.path().to_string_lossy()), true).await;
    let task = world.task(&project, "report-builder").await;
    let step = world.step(&task, "First", 0, TaskStepState::Running).await;
    world.services.step_snapshots.take(task.id, Some(step.id)).await;
    fs::write(repo.path().join("seed.txt"), "edited before and by the session").unwrap();

    let outcome = world.services.auto_commit.after_step_claim(task.id, step.id, None).await.unwrap();

    assert_eq!(outcome.status, AutoCommitStatus::Committed);
    assert_eq!(git(repo.path(), &["show", "--name-only", "--format=", "HEAD"]), "seed.txt");
    assert!(outcome.left_out.is_empty());
}

#[tokio::test]
async fn when_every_modified_file_was_there_before_nothing_is_committed_and_they_are_named() {
    let world = world().await;
    let repo = ready_repo().await;
    fs::write(repo.path().join("notes.txt"), "mine").unwrap();
    let project = world.project("vega", Some(&repo.path().to_string_lossy()), true).await;
    let task = world.task(&project, "report-builder").await;
    let step = world.step(&task, "First", 0, TaskStepState::Running).await;
    world.services.step_snapshots.take(task.id, Some(step.id)).await;

    let outcome = world.services.auto_commit.after_step_claim(task.id, step.id, None).await.unwrap();

    assert_eq!(outcome.status, AutoCommitStatus::Skipped);
    assert!(outcome.describe().contains("nothing of this work to commit"));
    assert!(outcome.describe().contains("notes.txt"));
    assert_eq!(git(repo.path(), &["log", "-1", "--format=%s"]), "seed");
    assert!(world.services.commit_references.find_all().await.unwrap().is_empty());
}

#[tokio::test]
async fn without_a_snapshot_everything_is_committed_and_the_answer_says_so() {
    let world = world().await;
    let repo = ready_repo().await;
    fs::write(repo.path().join("a.txt"), "a").unwrap();
    fs::write(repo.path().join("b.txt"), "b").unwrap();
    let project = world.project("vega", Some(&repo.path().to_string_lossy()), true).await;
    let task = world.task(&project, "report-builder").await;
    let step = world.step(&task, "First", 0, TaskStepState::Running).await;

    let outcome = world.services.auto_commit.after_step_claim(task.id, step.id, None).await.unwrap();

    assert_eq!(outcome.status, AutoCommitStatus::Committed);
    assert_eq!(git(repo.path(), &["show", "--name-only", "--format=", "HEAD"]), "a.txt\nb.txt");
    assert!(outcome.describe().contains("No snapshot of the folder from when this work started, so everything modified was committed."));
}

#[tokio::test]
async fn the_generated_message_is_typed_from_the_committed_files_only() {
    let world = world().await;
    let repo = ready_repo().await;
    fs::write(repo.path().join("unfinished.rs"), "fn main() {}").unwrap();
    let project = world.project("vega", Some(&repo.path().to_string_lossy()), true).await;
    let task = world.task(&project, "report-builder").await;
    let step = world.step(&task, "Describe the export", 0, TaskStepState::Running).await;
    world.services.step_snapshots.take(task.id, Some(step.id)).await;
    fs::write(repo.path().join("README.md"), "docs").unwrap();

    world.services.auto_commit.after_step_claim(task.id, step.id, None).await.unwrap();

    assert!(git(repo.path(), &["log", "-1", "--format=%s"]).starts_with("docs: "));
}

#[tokio::test]
async fn the_wrapup_of_a_task_with_no_steps_commits_only_what_changed_since_it_started() {
    let world = world().await;
    let repo = ready_repo().await;
    fs::write(repo.path().join("notes.txt"), "mine").unwrap();
    let project = world.project("vega", Some(&repo.path().to_string_lossy()), true).await;
    let task = world.task(&project, "fix-typo").await;
    world.services.step_snapshots.take(task.id, None).await;
    fs::write(repo.path().join("README.md"), "fixed").unwrap();

    let outcome = world.services.auto_commit.after_wrapup(task.id, None).await.unwrap();

    assert_eq!(outcome.status, AutoCommitStatus::Committed);
    assert_eq!(git(repo.path(), &["show", "--name-only", "--format=", "HEAD"]), "README.md");
    assert_eq!(outcome.left_out, vec!["notes.txt".to_string()]);
}

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
    assert_eq!(message, "feat: Wire the export endpoint\n\nRefs: task:REPORT-BUILDER, step 3");
    assert!(outcome.describe().contains(&format!("`{}`", &reference.commit_hash[..7])));
    assert!(outcome.describe().contains("\"Wire the export endpoint\""));
}

#[tokio::test]
async fn a_task_with_a_worktree_commits_on_its_branch_there_and_leaves_the_project_folder_alone() {
    let world = world().await;
    let repo = ready_repo().await;
    let project = world.project("vega", Some(&repo.path().to_string_lossy()), true).await;
    let task = world.task(&project, "report-builder").await;
    let step = world.step(&task, "Wire the export endpoint", 0, TaskStepState::Claimed).await;
    let worktree = repo.path().parent().unwrap().join(format!("worktree-{}", task.id));
    git(repo.path(), &["worktree", "add", "-q", "-b", "rekall/report-builder", &worktree.to_string_lossy()]);
    let mut row = task.clone().into_active_model();
    row.worktree_folder = sea_orm::ActiveValue::Set(Some(worktree.to_string_lossy().into_owned()));
    row.update(world.db()).await.unwrap();
    let tip_before = git(repo.path(), &["rev-parse", "HEAD"]);
    fs::write(worktree.join("export.ts"), "export").unwrap();

    let outcome = world.services.auto_commit.after_step_claim(task.id, step.id, None).await.unwrap();

    assert_eq!(outcome.status, AutoCommitStatus::Committed);
    assert_eq!(outcome.reference.unwrap().commit_hash, git(&worktree, &["rev-parse", "HEAD"]));
    assert_eq!(git(&worktree, &["rev-parse", "--abbrev-ref", "HEAD"]), "rekall/report-builder");
    assert_eq!(git(repo.path(), &["rev-parse", "HEAD"]), tip_before, "the project's own branch did not move");
    git(repo.path(), &["worktree", "remove", "--force", &worktree.to_string_lossy()]);
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
        "fix: stream the export\n\nIt no longer buffers the file.\n\nRefs: task:REPORT-BUILDER, step 3"
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
    assert!(references.recent_commits(task.id, None).await.unwrap_err().message().contains("folder"));
    let unknown = references.record_latest_commit(Id::random(), None).await.unwrap_err();
    assert!(matches!(unknown, rekall_kernel::RekallError::NotFound(_)));
    let wrong_step = references.record_latest_commit(task.id, Some(foreign.id)).await.unwrap_err();
    assert!(wrong_step.message().contains("does not belong"));
    let missing_label = references.record_commit_by_anchor(Some("vega"), "missing", None, None, None).await.unwrap_err();
    assert!(matches!(missing_label, rekall_kernel::RekallError::UnknownAnchor(_)));
    assert!(matches!(references.diff_for(Id::random()).await.unwrap_err(), rekall_kernel::RekallError::NotFound(_)));
    assert!(matches!(references.delete(Id::random()).await.unwrap_err(), rekall_kernel::RekallError::NotFound(_)));
    assert!(matches!(references.set_in_context(Id::random(), Some(true)).await.unwrap_err(), rekall_kernel::RekallError::NotFound(_)));
}

#[tokio::test]
async fn a_logged_commit_is_chosen_for_the_context_and_a_missing_flag_reads_as_off() {
    let world = world().await;
    let repo = ready_repo().await;
    let project = world.project("vega", Some(&repo.path().to_string_lossy()), false).await;
    let task = world.task(&project, "report-builder").await;
    let references = &world.services.commit_references;
    let logged = references.record_commit_by_anchor(Some("vega"), "report-builder", None, None, None).await.unwrap();

    assert!(references.set_in_context(logged.id, Some(true)).await.unwrap().in_context);
    assert!(!references.set_in_context(logged.id, None).await.unwrap().in_context);
    assert!(references.diff_for(logged.id).await.unwrap().unwrap().contains("seed"));
    references.delete(logged.id).await.unwrap();
    assert!(references.find_all().await.unwrap().is_empty());
    let _ = task;
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

    let entries = rekall_store::time_entry::Entity::find().all(world.db()).await.unwrap();
    assert_eq!(entries.len(), 2);
    assert!(entries.iter().all(|e| e.stopped_at.is_some()));
}

/// A world whose clock sits `hours` ahead of the rows the support helpers stamp, so those rows are idle.
async fn world_hours_ahead(hours: i64) -> support::World {
    world_at(Instant::now().plus(TimeDelta::hours(hours))).await
}

async fn open_entry_at(world: &support::World, task_id: Id, started_at: Instant) {
    let entry = rekall_store::time_entry::Model { id: Id::random(), task_id, started_at, stopped_at: None, created_at: started_at, updated_at: started_at };
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
    let entries = rekall_store::time_entry::Entity::find().all(world.db()).await.unwrap();
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
    let entries = rekall_store::time_entry::Entity::find().all(world.db()).await.unwrap();
    entries.iter().filter(|e| e.stopped_at.is_none()).count()
}

#[tokio::test]
async fn accepting_a_claimed_step_stops_the_tasks_timer() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "report").await;
    let step = world.step(&task, "export", 0, TaskStepState::Claimed).await;
    world.services.time_entries.start(task.id).await.unwrap();

    world.services.steps.edit(step.id, StepEdit { done: Some(true), ..StepEdit::default() }).await.unwrap();

    assert_eq!(running_entries(&world).await, 0);
}

#[tokio::test]
async fn ticking_an_open_step_by_hand_keeps_the_timer_running() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "report").await;
    let step = world.step(&task, "export", 0, TaskStepState::Open).await;
    world.services.time_entries.start(task.id).await.unwrap();

    world.services.steps.edit(step.id, StepEdit { done: Some(true), ..StepEdit::default() }).await.unwrap();

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

    let mut tx = rekall_core::Tx::write(&world.services.ctx).await.unwrap();
    world.services.review.accept_in(&mut tx, claimed.id).await.unwrap();
    world.services.review.accept_in(&mut tx, open.id).await.unwrap();
    tx.commit().await.unwrap();

    let entries = rekall_store::time_entry::Entity::find().all(world.db()).await.unwrap();
    let stopped = |task_id: Id| entries.iter().find(|e| e.task_id == task_id).unwrap().stopped_at.is_some();
    assert!(stopped(claimed.id));
    assert!(!stopped(open.id));
}

// ------------------------------------------------------------------------------ quick tasks

async fn quick_task(world: &support::World, project: &rekall_store::project::Model, label: &str) -> rekall_store::task::Model {
    let mut row = world.task(project, label).await;
    row.quick = true;
    row.clone().into_active_model().reset_all().update(world.db()).await.unwrap();
    row
}

async fn reloaded(world: &support::World, task_id: Id) -> rekall_store::task::Model {
    rekall_store::repository::task::require(world.db(), task_id).await.unwrap()
}

async fn wrapup_then_commit(world: &support::World, task: &str, author: rekall_store::WrapupAuthor) -> bool {
    let written = world.services.wrapups.write_by_anchor(Some("vega"), task, Some("State."), author).await.unwrap();
    let task_id = written.wrapup.task_id;
    let committed = world.services.auto_commit.after_wrapup(task_id, None).await.unwrap();
    world.services.wrapups.accept_quick_task_after_commit(task_id, &committed).await.unwrap()
}

#[tokio::test]
async fn the_wrapup_claude_writes_on_a_quick_task_accepts_it_and_marks_it_auto_accepted() {
    let world = world().await;
    let repo = ready_repo().await;
    fs::write(repo.path().join("README.md"), "fixed").unwrap();
    let project = world.project("vega", Some(&repo.path().to_string_lossy()), true).await;
    let task = quick_task(&world, &project, "fix-typo").await;

    assert!(wrapup_then_commit(&world, "fix-typo", rekall_store::WrapupAuthor::Claude).await);

    let after = reloaded(&world, task.id).await;
    assert_eq!(after.review_state, TaskStepState::Done);
    assert!(after.auto_accepted);
    assert!(after.accepted_at.is_some());
    assert!(git(repo.path(), &["log", "-1", "--format=%s"]).starts_with("docs:"));
}

#[tokio::test]
async fn a_quick_task_on_a_project_that_does_not_auto_commit_is_accepted_all_the_same() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = quick_task(&world, &project, "fix-typo").await;

    assert!(wrapup_then_commit(&world, "fix-typo", rekall_store::WrapupAuthor::Claude).await);

    assert_eq!(reloaded(&world, task.id).await.review_state, TaskStepState::Done);
}

#[tokio::test]
async fn a_quick_task_whose_commit_failed_stays_claimed_for_a_person() {
    let world = world().await;
    let plain = tempfile::tempdir().unwrap();
    let project = world.project("vega", Some(&plain.path().to_string_lossy()), true).await;
    let task = quick_task(&world, &project, "fix-typo").await;

    assert!(!wrapup_then_commit(&world, "fix-typo", rekall_store::WrapupAuthor::Claude).await);

    let after = reloaded(&world, task.id).await;
    assert_eq!(after.review_state, TaskStepState::Claimed);
    assert!(!after.auto_accepted);
}

#[tokio::test]
async fn a_wrapup_written_by_hand_does_not_accept_a_quick_task() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = quick_task(&world, &project, "fix-typo").await;

    assert!(!wrapup_then_commit(&world, "fix-typo", rekall_store::WrapupAuthor::Hand).await);

    assert_eq!(reloaded(&world, task.id).await.review_state, TaskStepState::Open);
}

#[tokio::test]
async fn a_task_that_is_not_quick_stays_claimed_after_claudes_wrapup() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "regular").await;

    assert!(!wrapup_then_commit(&world, "regular", rekall_store::WrapupAuthor::Claude).await);

    assert_eq!(reloaded(&world, task.id).await.review_state, TaskStepState::Claimed);
}

#[tokio::test]
async fn sending_an_auto_accepted_task_back_reopens_it_and_clears_the_mark() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = quick_task(&world, &project, "fix-typo").await;
    assert!(wrapup_then_commit(&world, "fix-typo", rekall_store::WrapupAuthor::Claude).await);

    let mut tx = rekall_core::Tx::write(&world.services.ctx).await.unwrap();
    world.services.review.send_back_in(&mut tx, task.id).await.unwrap();
    tx.commit().await.unwrap();

    let after = reloaded(&world, task.id).await;
    assert_eq!(after.review_state, TaskStepState::Open);
    assert!(!after.auto_accepted);
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

    let named = references.record_commit(task.id, None, Some(&format!(" {} ", &earlier[..7])), None).await.unwrap();
    assert_eq!(named.commit_hash, earlier);
    assert_eq!(named.comment, "seed");

    let blank = references.record_commit(task.id, None, Some("   "), None).await.unwrap();
    assert_eq!(blank.commit_hash, tip, "a blank hash is the plain button: the tip");
    assert_eq!(blank.comment, "The later one");

    let by_anchor = references.record_commit_by_anchor(Some("vega"), "report-builder", None, Some(&earlier), None).await.unwrap();
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

    let recent = world.services.commit_references.recent_commits(task.id, None).await.unwrap();

    let subjects: Vec<&str> = recent.iter().map(|entry| entry.subject.as_str()).collect();
    assert_eq!(subjects, ["The later one", "seed"]);
    assert_eq!(recent[0].hash, git(repo.path(), &["rev-parse", "HEAD"]));
}

#[tokio::test]
async fn a_tasks_changes_are_its_logged_commits_patches_oldest_first_and_the_working_tree() {
    let world = world().await;
    let repo = ready_repo().await;
    fs::write(repo.path().join("seed.txt"), "seed\nsecond\n").unwrap();
    git(repo.path(), &["add", "-A"]);
    commit(repo.path(), "Extend the seed");
    let first = git(repo.path(), &["rev-parse", "HEAD"]);
    let project = world.project("vega", Some(&repo.path().to_string_lossy()), false).await;
    let task = world.task(&project, "report-builder").await;
    world.services.commit_references.record_commit(task.id, None, Some(&first), None).await.unwrap();

    let clean = world.services.task_changes.of_task(task.id).await.unwrap();

    assert_eq!(clean.repositories.len(), 1);
    assert_eq!(clean.repositories[0].component_id, None);
    assert_eq!(clean.repositories[0].commits.len(), 1);
    let logged = &clean.repositories[0].commits[0];
    assert_eq!((logged.commit_hash.as_str(), logged.subject.as_str(), logged.available), (first.as_str(), "Extend the seed", true));
    assert_eq!(logged.files.len(), 1);
    assert_eq!((logged.files[0].path.as_str(), logged.files[0].additions, logged.files[0].deletions), ("seed.txt", 2, 1));
    assert!(clean.repositories[0].working_tree.is_none(), "a clean tree has no working-tree change");

    fs::write(repo.path().join("fresh.txt"), "new\n").unwrap();
    let dirty = world.services.task_changes.of_task(task.id).await.unwrap();

    let paths: Vec<&str> = dirty.repositories[0].working_tree.as_ref().unwrap().files.iter().map(|file| file.path.as_str()).collect();
    assert_eq!(paths, ["fresh.txt"]);
}

#[tokio::test]
async fn a_commit_git_no_longer_knows_is_read_from_the_diff_stored_when_it_was_logged() {
    let world = world().await;
    let repo = ready_repo().await;
    let project = world.project("vega", Some(&repo.path().to_string_lossy()), false).await;
    let task = world.task(&project, "report-builder").await;
    let stored = "diff --git a/gone.txt b/gone.txt\n--- a/gone.txt\n+++ b/gone.txt\n@@ -1 +1 @@\n-a\n+b\n";
    rekall_store::commit_reference::Model {
        id: Id::random(),
        task_id: task.id,
        step_id: None,
        component_id: None,
        commit_hash: "deadbeefdeadbeef".to_string(),
        comment: "Rebased away".to_string(),
        diff: Some(stored.to_string()),
        in_context: false,
        created_at: Instant::now(),
    }
    .into_active_model()
    .insert(world.db())
    .await
    .unwrap();

    let changes = world.services.task_changes.of_task(task.id).await.unwrap();

    assert!(changes.repositories[0].commits[0].available);
    assert_eq!(changes.repositories[0].commits[0].files[0].path, "gone.txt");
}

async fn add_component(world: &crate::support::World, project: &rekall_store::project::Model, name: &str, folder: &std::path::Path) -> rekall_store::project_component::Model {
    use sea_orm::{ActiveModelTrait, IntoActiveModel};
    let now = Instant::now();
    let component = rekall_store::project_component::Model {
        id: Id::random(), project_id: project.id, name: name.into(), folder: folder.to_string_lossy().into_owned(),
        description: String::new(), position: 0, created_at: now, updated_at: now,
    };
    component.clone().into_active_model().insert(world.db()).await.unwrap();
    component
}

#[tokio::test]
async fn changes_are_grouped_by_the_repository_each_commit_was_read_from() {
    let world = world().await;
    let own = ready_repo().await;
    let other = ready_repo().await;
    fs::write(other.path().join("site.txt"), "page\n").unwrap();
    git(other.path(), &["add", "-A"]);
    commit(other.path(), "Add the page");
    let project = world.project("vega", Some(&own.path().to_string_lossy()), false).await;
    let component = add_component(&world, &project, "Site", other.path()).await;
    let task = world.task(&project, "report-builder").await;
    let references = &world.services.commit_references;

    let logged = references.record_commit_by_anchor(Some("vega"), "report-builder", None, None, Some(" site ")).await.unwrap();
    references.record_commit(task.id, None, None, None).await.unwrap();
    let changes = world.services.task_changes.of_task(task.id).await.unwrap();

    assert_eq!((logged.component_id, logged.component_name.as_deref()), (Some(component.id), Some("Site")));
    let groups: Vec<_> = changes.repositories.iter().map(|group| (group.component_id, group.commits.len())).collect();
    assert_eq!(groups, [(None, 1), (Some(component.id), 1)]);
    assert_eq!(changes.repositories[1].name, "Site");
    assert_eq!(changes.repositories[1].commits[0].subject, "Add the page");
    assert_eq!(changes.repositories[1].commits[0].files[0].path, "site.txt");
}

#[tokio::test]
async fn a_commit_is_refused_for_a_component_that_is_not_of_the_project() {
    let world = world().await;
    let own = ready_repo().await;
    let elsewhere = ready_repo().await;
    let project = world.project("vega", Some(&own.path().to_string_lossy()), false).await;
    let other_project = world.project("orion", Some(&elsewhere.path().to_string_lossy()), false).await;
    let foreign = add_component(&world, &other_project, "Foreign", elsewhere.path()).await;
    let task = world.task(&project, "report-builder").await;

    let refused = world.services.commit_references.record_commit(task.id, None, None, Some(foreign.id)).await.unwrap_err();
    let unknown = world.services.commit_references.record_commit_by_anchor(Some("vega"), "report-builder", None, None, Some("Nope")).await.unwrap_err();

    assert!(refused.message().contains("does not belong"));
    assert!(unknown.message().contains("No component named 'Nope'"));
}

#[tokio::test]
async fn a_task_with_no_commit_and_no_folder_has_no_changes() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "report-builder").await;

    let changes = world.services.task_changes.of_task(task.id).await.unwrap();

    assert!(changes.repositories.is_empty());
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

#[tokio::test]
async fn a_search_in_a_project_finds_its_own_text_even_when_other_projects_have_eight_newer_hits() {
    use rekall_core::note::OnTitleClash;
    use rekall_store::WrapupAuthor;

    let world = world().await;
    let vega = world.project("vega", None, false).await;
    let orion = world.project("orion", None, false).await;
    world.task(&vega, "export").await;
    world.services.wrapups.write_by_anchor(Some("vega"), "export", Some("The settlement batch runs nightly."), WrapupAuthor::Claude).await.unwrap();
    for number in 0..9 {
        let label = format!("other{number}");
        world.task(&orion, &label).await;
        world.services.wrapups.write_by_anchor(Some("orion"), &label, Some("Another settlement batch."), WrapupAuthor::Claude).await.unwrap();
    }
    world.task(&orion, "ledger").await;
    world.services.notes.write(Some("orion"), "ledger", Some("ledger.md"), Some("Settlement of the ledger."), OnTitleClash::Refuse).await.unwrap();

    let scoped = world.services.search.search_in_project(Some("settlement"), "vega").await.unwrap();

    assert_eq!(scoped.len(), 1, "{scoped:?}");
    assert_eq!(scoped[0].r#where, "task:EXPORT");
    assert!(scoped[0].excerpt.contains("settlement batch runs nightly"));
    assert!(world.services.search.search(Some("settlement")).await.unwrap().len() > scoped.len());
    let in_orion = world.services.search.search_in_project(Some("settlement"), "orion").await.unwrap();
    assert!(in_orion.iter().any(|hit| hit.title == "ledger.md" && hit.context_anchor.starts_with("note:")));
}

#[tokio::test]
async fn a_search_in_an_unknown_project_is_refused() {
    let world = world().await;
    assert!(world.services.search.search_in_project(Some("settlement"), "nowhere").await.is_err());
}

// ------------------------------------------------------------------------------ wrapup versions

async fn write_wrapup(world: &support::World, task: &str, body: &str, author: rekall_store::WrapupAuthor) {
    world.services.wrapups.write_by_anchor(Some("vega"), task, Some(body), author).await.unwrap();
}

#[tokio::test]
async fn each_change_of_a_wrapup_text_keeps_a_version_newest_first_and_an_unchanged_text_keeps_none() {
    use rekall_store::WrapupAuthor::{Claude, Hand};

    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "export").await;

    write_wrapup(&world, "export", "First state.", Claude).await;
    write_wrapup(&world, "export", "Second state.", Hand).await;
    write_wrapup(&world, "export", "Second state.", Claude).await;

    let versions = world.services.wrapups.versions(task.id).await.unwrap();
    let kept: Vec<(i32, rekall_store::WrapupAuthor, usize)> = versions.iter().map(|v| (v.number, v.written_by, v.length)).collect();
    assert_eq!(kept, [(2, Hand, 13), (1, Claude, 12)]);
    let whole = world.services.wrapups.version(task.id, versions[1].id).await.unwrap();
    assert_eq!(whole.body_markdown, "First state.");
    assert_eq!(whole.version.preview, "First state.");
}

#[tokio::test]
async fn only_the_last_twenty_versions_stay() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "export").await;

    for number in 1..=21 {
        write_wrapup(&world, "export", &format!("State {number}."), rekall_store::WrapupAuthor::Claude).await;
    }

    let versions = world.services.wrapups.versions(task.id).await.unwrap();
    assert_eq!(versions.len(), 20);
    assert_eq!((versions[0].number, versions[19].number), (21, 2));
}

#[tokio::test]
async fn restoring_an_old_text_is_a_new_wrapup_and_a_new_version() {
    use rekall_store::WrapupAuthor::{Claude, Hand};

    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "export").await;
    write_wrapup(&world, "export", "Old state.", Claude).await;
    write_wrapup(&world, "export", "Bad rewrite.", Claude).await;
    let old = world.services.wrapups.versions(task.id).await.unwrap().pop().unwrap();
    let restored_text = world.services.wrapups.version(task.id, old.id).await.unwrap().body_markdown;

    world.services.wrapups.write(task.id, Some(&restored_text), Hand).await.unwrap();

    assert_eq!(world.services.wrapups.find(task.id).await.unwrap().unwrap().body_markdown, "Old state.");
    let versions = world.services.wrapups.versions(task.id).await.unwrap();
    assert_eq!((versions.len(), versions[0].number, versions[0].written_by), (3, 3, Hand));
}

#[tokio::test]
async fn deleting_a_wrapup_keeps_its_versions_and_a_version_of_another_task_is_not_found() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let export = world.task(&project, "export").await;
    let other = world.task(&project, "import").await;
    write_wrapup(&world, "export", "Export state.", rekall_store::WrapupAuthor::Claude).await;

    world.services.wrapups.delete(export.id).await.unwrap();

    let versions = world.services.wrapups.versions(export.id).await.unwrap();
    assert_eq!(versions.len(), 1);
    assert!(world.services.wrapups.version(other.id, versions[0].id).await.is_err());
}

#[tokio::test]
async fn a_version_records_the_commit_the_project_was_at() {
    let world = world().await;
    let repo = ready_repo().await;
    let project = world.project("vega", Some(&repo.path().to_string_lossy()), false).await;
    let task = world.task(&project, "export").await;
    world.project("orion", None, false).await;

    write_wrapup(&world, "export", "State.", rekall_store::WrapupAuthor::Claude).await;

    let versions = world.services.wrapups.versions(task.id).await.unwrap();
    assert_eq!(versions[0].commit_hash.as_deref(), Some(git(repo.path(), &["rev-parse", "HEAD"]).as_str()));
}

// ------------------------------------------------------------------------------ wrapup freshness

async fn stale_world(body: &str) -> (support::World, tempfile::TempDir, rekall_store::task::Model) {
    let world = world().await;
    let repo = ready_repo().await;
    let project = world.project("vega", Some(&repo.path().to_string_lossy()), false).await;
    let task = world.task(&project, "export").await;
    world.services.wrapups.write_by_anchor(Some("vega"), "export", Some(body), rekall_store::WrapupAuthor::Claude).await.unwrap();
    (world, repo, task)
}

fn commit_file(repo: &std::path::Path, name: &str, content: &str) -> String {
    if let Some(parent) = std::path::Path::new(name).parent() {
        fs::create_dir_all(repo.join(parent)).unwrap();
    }
    fs::write(repo.join(name), content).unwrap();
    git(repo, &["add", "-A"]);
    commit(repo, &format!("change {name}"));
    git(repo, &["rev-parse", "HEAD"])
}

#[tokio::test]
async fn a_commit_on_a_named_file_after_the_wrapup_makes_it_possibly_stale() {
    use rekall_core::wrapup::Freshness;
    let (world, repo, task) = stale_world("It lives in `a/b.rs`.").await;
    let changed = commit_file(repo.path(), "a/b.rs", "one");

    let freshness = world.services.wrapup_freshness.check(task.id).await.unwrap();

    assert_eq!(freshness, Freshness::PossiblyStale { changed_paths: vec!["a/b.rs".into()], commits: vec![changed] });
}

#[tokio::test]
async fn a_commit_on_another_file_leaves_it_fresh() {
    use rekall_core::wrapup::Freshness;
    let (world, repo, task) = stale_world("It lives in `a/b.rs`.").await;
    commit_file(repo.path(), "other.rs", "one");

    assert_eq!(world.services.wrapup_freshness.check(task.id).await.unwrap(), Freshness::Fresh);
}

#[tokio::test]
async fn a_commit_logged_against_the_task_itself_does_not_count() {
    use rekall_core::wrapup::Freshness;
    let (world, repo, task) = stale_world("It lives in `a/b.rs`.").await;
    let own = commit_file(repo.path(), "a/b.rs", "one");
    world.services.commit_references.record_commit(task.id, None, Some(&own), None).await.unwrap();

    assert_eq!(world.services.wrapup_freshness.check(task.id).await.unwrap(), Freshness::Fresh);
}

#[tokio::test]
async fn a_rewrite_starts_from_the_new_commit_and_the_warning_goes() {
    use rekall_core::wrapup::Freshness;
    let (world, repo, task) = stale_world("It lives in `a/b.rs`.").await;
    commit_file(repo.path(), "a/b.rs", "one");
    world.services.wrapups.write_by_anchor(Some("vega"), "export", Some("It lives in `a/b.rs`, rewritten."), rekall_store::WrapupAuthor::Claude).await.unwrap();

    assert_eq!(world.services.wrapup_freshness.check(task.id).await.unwrap(), Freshness::Fresh);
}

#[tokio::test]
async fn what_git_cannot_say_is_unknown_not_stale() {
    use rekall_core::wrapup::Freshness;
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "export").await;
    assert_eq!(world.services.wrapup_freshness.check(task.id).await.unwrap(), Freshness::Unknown, "no wrapup");
    world.services.wrapups.write_by_anchor(Some("vega"), "export", Some("It lives in `a/b.rs`."), rekall_store::WrapupAuthor::Claude).await.unwrap();
    assert_eq!(world.services.wrapup_freshness.check(task.id).await.unwrap(), Freshness::Unknown, "no repository");

    let (world, _repo, task) = stale_world("It names nothing at all.").await;
    assert_eq!(world.services.wrapup_freshness.check(task.id).await.unwrap(), Freshness::Unknown, "no path named");

    let many: String = (0..21).map(|n| format!("`d/f{n}.rs` ")).collect();
    let (world, _repo, task) = stale_world(&many).await;
    assert_eq!(world.services.wrapup_freshness.check(task.id).await.unwrap(), Freshness::Unknown, "over twenty paths");
}

#[tokio::test]
async fn the_context_of_a_task_carries_the_warning_on_its_wrapup() {
    let (world, repo, task) = stale_world("It lives in `a/b.rs`.").await;
    commit_file(repo.path(), "a/b.rs", "one");

    let rendered = world.services.renderer.render(&world.services.context.for_task(task.id).await.unwrap());

    assert!(rendered.contains("freshness=\"possibly stale: a/b.rs changed in 1 commit since it was written\""), "{rendered}");
}

// ------------------------------------------------------------------------------ send back

#[tokio::test]
async fn a_step_sent_back_hands_the_next_session_its_earlier_pass_and_the_feedback_apart() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "report-builder").await;
    let step = world.step(&task, "Aggregate the rows", 0, TaskStepState::Open).await;
    let steps = &world.services.steps;
    steps.edit(step.id, StepEdit { body_markdown: Some("Sum by month"), ..StepEdit::default() }).await.unwrap();
    steps.transition(Some("vega"), "report-builder", Some("1"), TaskStepState::Claimed).await.unwrap();

    let sent_back = steps.send_back(step.id).await.unwrap();
    assert_eq!(sent_back.state, TaskStepState::Open);
    assert_eq!(sent_back.body_markdown, None);
    assert_eq!(sent_back.passes[0].detail_markdown.as_deref(), Some("Sum by month"));

    steps.edit(step.id, StepEdit { body_markdown: Some("Group by week instead"), ..StepEdit::default() }).await.unwrap();
    let record = world.services.context.load_task("vega", "report-builder").await.unwrap();
    let rendered = world.services.renderer.render_steps(&record.steps, &record.step_connections, None, &mut ComponentReferences::new(None));

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

// ------------------------------------------------------------------------------ component references

#[tokio::test]
async fn a_component_named_in_a_task_text_is_expanded_in_the_context_but_not_in_the_database() {
    use rekall_store::project_component;
    use sea_orm::{ActiveModelTrait, ActiveValue};

    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "report-builder").await;
    let now = Instant::now();
    project_component::Model {
        id: Id::random(),
        project_id: project.id,
        name: "Auth Service".into(),
        folder: "services/auth".into(),
        description: "Issues tokens.".into(),
        position: 0,
        created_at: now,
        updated_at: now,
    }
    .into_active_model()
    .insert(world.db())
    .await
    .unwrap();
    let mut row = task.clone().into_active_model();
    row.description = ActiveValue::Set(Some("Call [[auth service]] and [[Billing]].".into()));
    row.update(world.db()).await.unwrap();
    let step = world.step(&task, "Check the tokens", 0, TaskStepState::Open).await;
    world.services.steps.edit(step.id, StepEdit { body_markdown: Some("Again [[Auth Service]]"), ..StepEdit::default() }).await.unwrap();

    let records = world.services.context.for_task(task.id).await.unwrap();
    let rendered = world.services.renderer.render(&records);

    assert!(rendered.contains("Call **Auth Service** (component, folder `services/auth`): Issues tokens. and [[Billing]] (component not found in this project)."), "{rendered}");
    assert!(rendered.contains("Again **Auth Service** (component, folder `services/auth`)\n"), "{rendered}");
    let stored = world.services.context.for_task(task.id).await.unwrap();
    assert_eq!(stored[1].description.as_deref(), Some("Call [[auth service]] and [[Billing]]."));
    let size = world.services.context_size.measure(task.id).await.unwrap();
    let description = size.parts.iter().find(|part| part.label == "Description").unwrap();
    assert!(description.characters > "Call [[auth service]] and [[Billing]].".len(), "{description:?}");
}

#[tokio::test]
async fn a_task_named_in_a_task_text_is_loaded_once_as_background_without_its_checklist() {
    use sea_orm::{ActiveModelTrait, ActiveValue};

    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "report-builder").await;
    let export = world.task(&project, "export").await;
    let mut row = export.clone().into_active_model();
    row.description = ActiveValue::Set(Some("Writes the CSV export.".into()));
    row.update(world.db()).await.unwrap();
    world.step(&export, "Stream the rows", 0, TaskStepState::Open).await;
    let mut row = task.clone().into_active_model();
    row.description = ActiveValue::Set(Some("Reuse [[task:export]] and [[task:missing]].".into()));
    row.update(world.db()).await.unwrap();
    let step = world.step(&task, "Wire the button", 0, TaskStepState::Open).await;
    world.services.steps.edit(step.id, StepEdit { body_markdown: Some("Like [[task:export]]"), ..StepEdit::default() }).await.unwrap();

    let records = world.services.context.for_task(task.id).await.unwrap();
    let rendered = world.services.renderer.render(&records);

    assert!(rendered.contains("Reuse **Title of export** (task `task:EXPORT`) and [[task:missing]] (task not found in this project)."), "{rendered}");
    assert_eq!(rendered.matches("### Task: Title of export").count(), 1, "{rendered}");
    assert!(rendered.contains("Writes the CSV export."), "{rendered}");
    assert!(rendered.contains("- `referenced`: named in the text of the task above."), "{rendered}");
    assert!(rendered.contains("- `steps`: 0 of 1 done, 1 open"), "{rendered}");
    assert!(!rendered.contains("Stream the rows"), "{rendered}");
    let size = world.services.context_size.measure(task.id).await.unwrap();
    assert!(size.parts.iter().any(|part| part.label == "Referenced task: Title of export"), "{size:?}");
}

#[tokio::test]
async fn a_task_of_an_epic_inherits_its_description_and_notes_and_the_epic_lists_its_tasks() {
    use rekall_store::TaskKind;
    use rekall_core::note::OnTitleClash;
    use sea_orm::ActiveValue;

    let world = world().await;
    let project = world.project("vega", None, false).await;
    let epic = world.task(&project, "checkout").await;
    let mut row = epic.clone().into_active_model();
    row.kind = ActiveValue::Set(TaskKind::Epic);
    row.description = ActiveValue::Set(Some("Every payment goes through one ledger.".into()));
    row.update(world.db()).await.unwrap();
    world.services.notes.write(Some("vega"), "checkout", Some("ledger.md"), Some("The ledger rules."), OnTitleClash::Refuse).await.unwrap();
    let task = world.task(&project, "pay").await;
    let mut row = task.clone().into_active_model();
    row.epic_id = ActiveValue::Set(Some(epic.id));
    row.update(world.db()).await.unwrap();

    let rendered = world.services.renderer.render(&world.services.context.for_task(task.id).await.unwrap());
    assert!(rendered.contains("- `epic`: belongs to the epic `task:CHECKOUT` below"), "{rendered}");
    assert!(rendered.contains("### Epic: Title of checkout"), "{rendered}");
    assert!(rendered.contains("Every payment goes through one ledger."), "{rendered}");
    assert!(rendered.contains("The ledger rules."), "{rendered}");

    let rendered = world.services.renderer.render(&world.services.context.for_task(epic.id).await.unwrap());
    assert!(rendered.contains("## Epic: Title of checkout"), "{rendered}");
    assert!(rendered.contains("1 task belongs to it (0 done)"), "{rendered}");
    assert!(rendered.contains("  - `task:PAY`"), "{rendered}");
}

#[tokio::test]
async fn a_task_context_lists_the_tasks_it_waits_for_and_where_each_stands() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let report = world.task(&project, "report").await;
    let export = world.task(&project, "export").await;
    rekall_store::task_dependency::Model::new(export.id, report.id, Instant::now()).into_active_model().insert(world.db()).await.unwrap();

    let rendered = world.services.renderer.render(&world.services.context.for_task(export.id).await.unwrap());

    assert!(rendered.contains("- `depends-on`: `task:REPORT` (not accepted)."), "{rendered}");
    assert!(rendered.contains("The run queue starts this task only once all are accepted."), "{rendered}");
}

// ------------------------------------------------------------------------------ task creation

fn draft(title: &str) -> rekall_core::flow::PlannedStep {
    rekall_core::flow::PlannedStep {
        title: title.into(),
        detail: Some(format!("Detail of {title}")),
        routes: vec![],
        max_visits: None,
        starts_session: false,
    }
}

fn plan(steps: Vec<rekall_core::flow::PlannedStep>) -> rekall_core::flow::FlowPlan {
    rekall_core::flow::FlowPlan::new(steps)
}

fn routed(title: &str, routes: &[(&str, Option<usize>)]) -> rekall_core::flow::PlannedStep {
    let routes = routes.iter().map(|(outcome, to)| rekall_core::flow::PlannedRoute { outcome: (*outcome).into(), to: *to }).collect();
    rekall_core::flow::PlannedStep { routes, ..draft(title) }
}

fn brief(title: &str, description: Option<&str>) -> rekall_core::task::TaskBrief {
    rekall_core::task::TaskBrief {
        title: Some(title.into()),
        description: description.map(str::to_string),
        kind: rekall_store::TaskKind::Task,
        epic: None,
    }
}

fn epic_brief(title: &str) -> rekall_core::task::TaskBrief {
    rekall_core::task::TaskBrief { kind: rekall_store::TaskKind::Epic, ..brief(title, None) }
}

fn brief_under(title: &str, epic: &str) -> rekall_core::task::TaskBrief {
    rekall_core::task::TaskBrief { epic: Some(epic.into()), ..brief(title, None) }
}

#[tokio::test]
async fn a_session_creates_an_epic_and_files_a_new_task_under_it() {
    let world = world().await;
    world.project("vega", None, false).await;

    let epic = world.services.task_authoring.create("vega", &epic_brief("Checkout"), &plan(vec![])).await.unwrap();
    world.services.task_authoring.create("vega", &brief_under("Pay", "VE-1"), &plan(vec![draft("Charge")])).await.unwrap();

    assert_eq!(epic.kind, rekall_store::TaskKind::Epic);
    assert_eq!(epic.task_anchor, "task:VE-1");
    let epic = rekall_store::repository::task::find_by_code(world.db(), "VE-1").await.unwrap().unwrap();
    let pay = rekall_store::repository::task::find_by_code(world.db(), "VE-2").await.unwrap().unwrap();
    assert!(epic.is_epic());
    assert_eq!(pay.epic_id, Some(epic.id));
}

#[tokio::test]
async fn an_epic_created_with_steps_or_under_a_plain_task_is_refused_and_creates_nothing() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    world.task(&project, "report").await;

    let with_steps = world.services.task_authoring.create("vega", &epic_brief("Checkout"), &plan(vec![draft("First")])).await.unwrap_err();
    let under_plain = world.services.task_authoring.create("vega", &brief_under("Pay", "report"), &plan(vec![])).await.unwrap_err();

    assert!(with_steps.to_string().contains("An epic holds no steps"), "{with_steps}");
    assert!(under_plain.to_string().contains("is not an epic"), "{under_plain}");
    assert_eq!(rekall_store::repository::task::list_of_project(world.db(), project.id).await.unwrap().len(), 1);
}

#[tokio::test]
async fn a_session_files_an_existing_task_under_an_epic_and_moves_it_between_epics() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    world.services.task_authoring.create("vega", &epic_brief("Checkout"), &plan(vec![])).await.unwrap();
    world.services.task_authoring.create("vega", &epic_brief("Billing"), &plan(vec![])).await.unwrap();
    world.task(&project, "pay").await;

    let joined = world.services.epic_membership.join(Some("vega"), "pay", "VE-1").await.unwrap();
    let moved = world.services.epic_membership.join(Some("vega"), "pay", "VE-2").await.unwrap();

    assert_eq!(joined.epic_anchor, "task:VE-1");
    assert_eq!(joined.left, None);
    assert_eq!(moved.left.as_deref(), Some("VE-1"));
    let pay = rekall_store::repository::task::find_by_code(world.db(), "PAY").await.unwrap().unwrap();
    let billing = rekall_store::repository::task::find_by_code(world.db(), "VE-2").await.unwrap().unwrap();
    assert_eq!(pay.epic_id, Some(billing.id));
}

#[tokio::test]
async fn an_epic_is_never_run() {
    let world = world().await;
    world.project("vega", Some("/tmp"), false).await;
    world.services.task_authoring.create("vega", &epic_brief("Checkout"), &plan(vec![])).await.unwrap();
    let epic = rekall_store::repository::task::find_by_code(world.db(), "VE-1").await.unwrap().unwrap();

    let refused = world.services.terminal_launch.resolve(rekall_core::claude::TerminalTarget::Task(epic.id)).await.unwrap_err();

    assert!(refused.to_string().contains("an epic is never run"), "{refused}");
}

#[tokio::test]
async fn a_created_task_starts_todo_with_its_description_and_its_steps_as_drafts_in_order() {
    let world = world().await;
    let project = world.project("vega", None, false).await;

    let created = world
        .services
        .task_authoring
        .create("vega", &brief("  Task creation ", Some("Let the llm open tasks.")), &plan(vec![draft("First"), draft("Second")]))
        .await
        .unwrap();

    assert_eq!(created.task_anchor, "task:VE-1");
    assert_eq!((created.steps, created.routes), (2, 0));
    let task = rekall_store::repository::task::find_by_code(world.db(), "VE-1").await.unwrap().unwrap();
    assert_eq!(task.project_id, project.id);
    assert_eq!(task.title, "Task creation");
    assert_eq!(task.status, rekall_store::TaskStatus::Todo);
    assert_eq!(task.description.as_deref(), Some("Let the llm open tasks."));
    let steps = world.services.steps.find_by_task(task.id).await.unwrap();
    let titles: Vec<&str> = steps.iter().map(|step| step.title.as_str()).collect();
    assert_eq!(titles, ["First", "Second"]);
    assert!(steps.iter().all(|step| step.state == TaskStepState::Open));
}

#[tokio::test]
async fn each_task_of_a_project_takes_the_next_number_after_the_last() {
    let world = world().await;
    world.project("vega", None, false).await;

    let first = world.services.task_authoring.create("vega", &brief("Report builder", None), &plan(vec![draft("First")])).await.unwrap();
    let second = world.services.task_authoring.create("vega", &brief("Report builder", None), &plan(vec![draft("First")])).await.unwrap();

    assert_eq!((first.task_anchor.as_str(), second.task_anchor.as_str()), ("task:VE-1", "task:VE-2"));
}

#[tokio::test]
async fn a_task_is_named_by_its_code_in_any_case_and_a_project_beside_it_must_be_its_own() {
    let world = world().await;
    world.project("vega", None, false).await;
    world.project("beacon", None, false).await;
    world.services.task_authoring.create("vega", &brief("Report builder", None), &plan(vec![])).await.unwrap();

    let context = &world.services.context;
    let found = context.load(Some(rekall_core::context::AnchorEntity::Task), "ve-1").await.unwrap();
    let with_project = context.load_task("VEGA", "VE-1").await.unwrap();
    let wrong_project = context.load_task("beacon", "VE-1").await.unwrap_err();
    let unknown = context.load(Some(rekall_core::context::AnchorEntity::Task), "VE-9").await.unwrap_err();

    assert_eq!((found.anchor.as_str(), with_project.anchor.as_str()), ("task:VE-1", "task:VE-1"));
    assert!(wrong_project.to_string().contains("No task 'VE-1' on project 'beacon'"), "{wrong_project}");
    assert!(unknown.to_string().contains("A task is named by its code"), "{unknown}");
}

#[tokio::test]
async fn an_anchor_written_with_a_project_and_a_label_before_codes_existed_still_finds_its_task() {
    use sea_orm::{ActiveModelTrait, ActiveValue};

    let world = world().await;
    world.project("vega", None, false).await;
    world.services.task_authoring.create("vega", &brief("Report builder", None), &plan(vec![])).await.unwrap();
    let task = rekall_store::repository::task::find_by_code(world.db(), "VE-1").await.unwrap().unwrap();
    let mut row = task.clone().into_active_model();
    row.label = ActiveValue::Set("report-builder".into());
    row.update(world.db()).await.unwrap();

    let by_label = world.services.context.load_task("vega", "report-builder").await.unwrap();
    let without_project = world.services.context.load(Some(rekall_core::context::AnchorEntity::Task), "report-builder").await.unwrap_err();

    assert_eq!(by_label.anchor, task.anchor());
    assert!(without_project.to_string().contains("No task 'report-builder'"), "{without_project}");
}

#[tokio::test]
async fn a_step_title_sent_twice_or_past_the_draft_cap_is_refused_before_anything_is_written() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let twice = plan(vec![draft("Same"), draft("same")]);
    let too_many = plan((0..21).map(|at| draft(&format!("Step {at}"))).collect());

    for steps in [&twice, &too_many] {
        let result = world.services.task_authoring.create("vega", &brief("New", None), steps).await;
        assert!(result.is_err());
    }

    assert!(rekall_store::repository::task::list_of_project(world.db(), project.id).await.unwrap().is_empty());
}

#[tokio::test]
async fn a_task_created_with_routes_is_a_flow_whose_open_steps_keep_their_sessions_and_visit_budgets() {
    let world = world().await;
    world.project("vega", None, false).await;
    let steps = vec![
        routed("Test", &[("passed", None), ("app-error", Some(1))]),
        rekall_core::flow::PlannedStep {
            max_visits: Some(rekall_core::step::VisitBudget::new(3).unwrap()),
            starts_session: true,
            ..routed("Fix", &[("fixed", Some(0))])
        },
    ];

    let created = world.services.task_authoring.create("vega", &brief("Release", None), &plan(steps)).await.unwrap();

    assert_eq!((created.steps, created.routes), (2, 3));
    let task = rekall_store::repository::task::find_by_code(world.db(), "VE-1").await.unwrap().unwrap();
    let stored = world.services.steps.find_by_task(task.id).await.unwrap();
    assert!(stored.iter().all(|step| step.state == TaskStepState::Open));
    assert_eq!(stored.iter().map(|step| step.starts_session).collect::<Vec<_>>(), [false, true]);
    assert_eq!(stored[1].max_visits, Some(3));
    assert_eq!(stored[0].links.len(), 2);
}

#[tokio::test]
async fn a_flow_that_could_loop_forever_is_refused_at_creation_and_creates_nothing() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let steps = vec![routed("Test", &[("app-error", Some(1))]), routed("Fix", &[("fixed", Some(0))])];

    let refused = world.services.task_authoring.create("vega", &brief("Release", None), &plan(steps)).await.unwrap_err();

    assert!(refused.to_string().contains("can loop forever"), "{refused}");
    assert!(rekall_store::repository::task::list_of_project(world.db(), project.id).await.unwrap().is_empty());
}

#[tokio::test]
async fn a_proposed_step_joins_the_running_session_unless_it_asks_for_a_new_one() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "report-builder").await;

    let joining = world.services.steps.propose(Some("vega"), "report-builder", NewStep::titled("Wire", None), Staging::Draft).await.unwrap();
    let opening = world.services.steps.propose(Some("vega"), "report-builder", NewStep { starts_session: true, ..NewStep::titled("Docs", None) }, Staging::Draft).await.unwrap();

    assert!(!joining.step.starts_session);
    assert!(opening.step.starts_session);
    let stored = world.services.steps.find_by_task(task.id).await.unwrap();
    assert_eq!(stored.iter().map(|step| step.starts_session).collect::<Vec<_>>(), [false, true]);
}

#[tokio::test]
async fn a_step_added_ready_is_open_and_sits_ahead_of_the_drafts_left_on_the_shelf() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "report-builder").await;
    world.services.steps.propose(Some("vega"), "report-builder", NewStep::titled("Review me", None), Staging::Draft).await.unwrap();

    let added = world.services.steps.propose(Some("vega"), "report-builder", NewStep::titled("Wire", None), Staging::Ready).await.unwrap();

    assert_eq!(added.draft_count, 1);
    let stored = world.services.steps.find_by_task(task.id).await.unwrap();
    assert_eq!(stored.iter().map(|step| (step.title.as_str(), step.state)).collect::<Vec<_>>(), [
        ("Wire", TaskStepState::Open),
        ("Review me", TaskStepState::Draft)
    ]);
}

#[tokio::test]
async fn a_step_can_be_made_to_open_a_session_and_made_to_join_one_again() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "report-builder").await;
    let step = world.step(&task, "Docs", 0, TaskStepState::Open).await;

    let opened = world.services.step_bindings.begin_session(step.id, true).await.unwrap();
    assert!(opened[0].starts_session);
    let joined = world.services.step_bindings.begin_session(step.id, false).await.unwrap();
    assert!(!joined[0].starts_session);
}

#[tokio::test]
async fn a_task_needs_a_title_and_a_project_that_exists() {
    let world = world().await;
    world.project("vega", None, false).await;

    let untitled = world.services.task_authoring.create("vega", &brief("   ", None), &plan(vec![])).await.unwrap_err();
    let unknown = world.services.task_authoring.create("nowhere", &brief("New", None), &plan(vec![])).await.unwrap_err();

    assert!(untitled.to_string().contains("needs a title"), "{untitled}");
    assert!(unknown.to_string().contains("No project matches 'nowhere'"), "{unknown}");
}

// --- Sessions

async fn claimed_step(world: &support::World, task: &rekall_store::task::Model, title: &str, position: i32) {
    let mut step = world.step(task, title, position, TaskStepState::Claimed).await;
    step.claimed_at = Some(Instant::now());
    step.clone().into_active_model().reset_all().update(world.db()).await.unwrap();
}

#[tokio::test]
async fn a_task_reads_as_spent_when_the_next_open_step_opens_a_session_of_its_own() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "report-builder").await;
    claimed_step(&world, &task, "Move", 0).await;
    let next = world.step(&task, "Docs", 1, TaskStepState::Open).await;

    assert!(!world.services.task_work.read(task.id).await.unwrap().next_opens_session);
    world.services.step_bindings.begin_session(next.id, true).await.unwrap();

    let work = world.services.task_work.read(task.id).await.unwrap();
    assert!(work.next_opens_session);
    assert!(work.last_settled_at.is_some());
    assert!(work.session_is_spent(&Instant::now().minus(TimeDelta::minutes(5))));
}

#[tokio::test]
async fn the_first_step_opening_a_session_is_no_boundary() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "report-builder").await;
    let first = world.step(&task, "Move", 0, TaskStepState::Open).await;
    world.services.step_bindings.begin_session(first.id, true).await.unwrap();

    assert!(!world.services.task_work.read(task.id).await.unwrap().next_opens_session);
}

#[tokio::test]
async fn a_claim_completes_a_step_and_a_task_with_only_claimed_steps_has_no_work_left() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "report-builder").await;
    claimed_step(&world, &task, "Move", 0).await;
    world.step(&task, "Docs", 1, TaskStepState::Open).await;
    assert!(world.services.task_work.read(task.id).await.unwrap().has_work());

    world.services.steps.transition(Some("vega"), "report-builder", Some("Docs"), TaskStepState::Claimed).await.unwrap();

    let work = world.services.task_work.read(task.id).await.unwrap();
    assert!(!work.has_work());
    assert_eq!(work.remaining, rekall_core::claude::Remaining::Claimed);
    assert_eq!(work.settled_steps, 2);
}

#[tokio::test]
async fn a_task_without_steps_is_complete_once_its_review_line_is_claimed() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "report-builder").await;
    assert!(world.services.task_work.read(task.id).await.unwrap().has_work());

    let mut claimed = task.clone();
    claimed.review_state = TaskStepState::Claimed;
    claimed.into_active_model().reset_all().update(world.db()).await.unwrap();

    let work = world.services.task_work.read(task.id).await.unwrap();
    assert!(!work.has_work());
    assert_eq!(work.remaining, rekall_core::claude::Remaining::Claimed);
}

#[tokio::test]
async fn a_deviation_is_kept_apart_from_the_notes_rewritten_for_the_same_step_and_loaded_with_the_context() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "checkout").await;
    world.step(&task, "Draw the ledger", 0, TaskStepState::Done).await;
    world.step(&task, "Wire the service", 1, TaskStepState::Open).await;
    world.step(&task, "Not yet promoted", 2, TaskStepState::Draft).await;

    let first = world.services.deviations.write(Some("vega"), "checkout", 2, Some("Planned a new layer; built it in core.")).await.unwrap();
    let second = world.services.deviations.write(Some("vega"), "checkout", 2, Some("Built it in core, one method.")).await.unwrap();

    assert_eq!((first.replaced, first.deviations_on_task, first.step_title.as_str()), (false, 1, "Wire the service"));
    assert_eq!((second.replaced, second.deviations_on_task), (true, 1));
    assert!(rekall_store::repository::document::documents_of_task(world.db(), task.id).await.unwrap().is_empty());
    let rendered = world.services.renderer.render(&world.services.context.for_task(task.id).await.unwrap());
    assert!(rendered.contains("<deviation step=\"2\" title=\"Wire the service\">\nBuilt it in core, one method.\n</deviation>"), "{rendered}");
    assert!(!rendered.contains("Planned a new layer"), "{rendered}");
}

#[tokio::test]
async fn a_deviation_for_a_step_the_checklist_does_not_have_is_refused_and_nothing_is_written() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "checkout").await;
    world.step(&task, "Draw the ledger", 0, TaskStepState::Open).await;
    world.step(&task, "Not yet promoted", 1, TaskStepState::Draft).await;

    for step_number in [0, 2, 9] {
        let refusal = world.services.deviations.write(Some("vega"), "checkout", step_number, Some("Changed.")).await.unwrap_err();
        assert!(refusal.message().contains("checklist has 1 step: there is no step"), "{refusal:?}");
    }
    assert!(world.services.deviations.write(Some("vega"), "checkout", 1, Some("   ")).await.is_err());
    assert!(rekall_store::repository::task_deviation::list_of_task(world.db(), task.id).await.unwrap().is_empty());
}
