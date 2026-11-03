//! `GitWorktreeHost` against a real repository in a temporary folder: a worktree is made on a
//! new branch of its own, kept apart from the project's files, listed, inspected, removed with
//! its branch kept, and nothing outside the worktree folder is ever touched.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use rekall_core::worktree::{OrphanReclaimService, WorktreeHost, WorktreeLayout};
use rekall_infra::worktree::GitWorktreeHost;

struct World {
    _folder: tempfile::TempDir,
    project: PathBuf,
    layout: WorktreeLayout,
    host: GitWorktreeHost,
}

fn git(folder: &Path, arguments: &[&str]) -> String {
    let output = Command::new("git").arg("-C").arg(folder).args(arguments).output().unwrap();
    assert!(output.status.success(), "git {arguments:?}: {}", String::from_utf8_lossy(&output.stderr));
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn world() -> World {
    let folder = tempfile::tempdir().unwrap();
    let project = folder.path().join("vega");
    fs::create_dir_all(&project).unwrap();
    git(&project, &["init", "-q", "-b", "main"]);
    git(&project, &["config", "user.email", "test@example.com"]);
    git(&project, &["config", "user.name", "Test"]);
    fs::write(project.join("report.txt"), "report\n").unwrap();
    git(&project, &["add", "."]);
    git(&project, &["commit", "-q", "-m", "first"]);
    let layout = WorktreeLayout::new(folder.path().join("data").join("worktrees"));
    World { _folder: folder, project, host: GitWorktreeHost::new(layout.clone()), layout }
}

async fn create(world: &World, task: &str) -> rekall_core::worktree::Worktree {
    let branch = world.layout.branch_for(task).unwrap();
    let folder = world.layout.folder_for("vega", task).unwrap();
    world.host.create(&world.project, &branch, &folder).await.unwrap()
}

#[tokio::test]
async fn a_worktree_is_a_folder_on_a_new_branch_that_leaves_the_project_untouched() {
    let world = world();

    let worktree = create(&world, "report").await;

    assert!(worktree.folder.join("report.txt").exists());
    assert_eq!(git(&worktree.folder, &["rev-parse", "--abbrev-ref", "HEAD"]), "rekall/report");
    fs::write(worktree.folder.join("export.txt"), "export\n").unwrap();
    assert!(!world.project.join("export.txt").exists());
    assert_eq!(git(&world.project, &["rev-parse", "--abbrev-ref", "HEAD"]), "main");
}

#[tokio::test]
async fn a_worktree_starts_from_the_current_head_of_the_project() {
    let world = world();
    fs::write(world.project.join("later.txt"), "later\n").unwrap();
    git(&world.project, &["add", "."]);
    git(&world.project, &["commit", "-q", "-m", "second"]);

    let worktree = create(&world, "report").await;

    assert!(worktree.folder.join("later.txt").exists());
}

#[tokio::test]
async fn removing_a_worktree_takes_the_folder_and_keeps_the_branch() {
    let world = world();
    let worktree = create(&world, "report").await;

    world.host.remove(&worktree).await.unwrap();

    assert!(!worktree.folder.exists());
    assert_eq!(git(&world.project, &["branch", "--list", "rekall/report"]).trim(), "rekall/report");
}

#[tokio::test]
async fn a_removed_worktree_comes_back_on_the_branch_with_its_commits() {
    let world = world();
    let first = create(&world, "report").await;
    fs::write(first.folder.join("export.txt"), "export\n").unwrap();
    git(&first.folder, &["add", "."]);
    git(&first.folder, &["commit", "-q", "-m", "export"]);
    world.host.remove(&first).await.unwrap();

    let again = create(&world, "report").await;

    assert!(again.folder.join("export.txt").exists());
    assert_eq!(git(&again.folder, &["rev-parse", "--abbrev-ref", "HEAD"]), "rekall/report");
}

#[tokio::test]
async fn the_listing_holds_the_worktrees_under_the_root_and_not_the_project_itself() {
    let world = world();
    let report = create(&world, "report").await;
    git(&world.project, &["worktree", "add", "-q", "-b", "rekall/elsewhere", world.project.parent().unwrap().join("elsewhere").to_str().unwrap()]);

    let listed = world.host.list(&world.project).await.unwrap();

    assert_eq!(listed, vec![report]);
}

#[tokio::test]
async fn the_status_tells_unsaved_changes_and_commits_ahead() {
    let world = world();
    let worktree = create(&world, "report").await;
    let clean = world.host.status(&worktree).await.unwrap();
    fs::write(worktree.folder.join("export.txt"), "export\n").unwrap();
    let unsaved = world.host.status(&worktree).await.unwrap();
    git(&worktree.folder, &["add", "."]);
    git(&worktree.folder, &["commit", "-q", "-m", "export"]);
    let committed = world.host.status(&worktree).await.unwrap();

    assert_eq!((clean.has_unsaved_changes, clean.commits_ahead), (false, 0));
    assert_eq!((unsaved.has_unsaved_changes, unsaved.commits_ahead), (true, 0));
    assert_eq!((committed.has_unsaved_changes, committed.commits_ahead), (false, 1));
}

#[tokio::test]
async fn a_branch_or_folder_outside_the_layout_is_refused_and_nothing_is_made() {
    let world = world();
    let inside = world.layout.folder_for("vega", "report").unwrap();
    let outside = world.project.parent().unwrap().join("stray");

    let foreign_branch = world.host.create(&world.project, "feature/report", &inside).await;
    let outside_folder = world.host.create(&world.project, "rekall/report", &outside).await;
    let escaping = world.host.create(&world.project, "rekall/report", &world.layout.root.join("vega").join("..").join("..").join("stray")).await;

    assert!(foreign_branch.is_err() && outside_folder.is_err() && escaping.is_err());
    assert!(!outside.exists() && !inside.exists());
    assert_eq!(git(&world.project, &["branch", "--list", "rekall/*"]), "");
}

#[tokio::test]
async fn removing_a_folder_outside_the_layout_is_refused() {
    let world = world();
    let victim = world.project.parent().unwrap().join("precious");
    fs::create_dir_all(&victim).unwrap();
    let forged = rekall_core::worktree::Worktree { project_folder: world.project.clone(), folder: victim.clone(), branch: "rekall/report".into() };

    assert!(world.host.remove(&forged).await.is_err());
    assert!(victim.exists());
}

#[tokio::test]
async fn a_folder_that_is_no_repository_is_refused_with_a_message() {
    let world = world();
    let not_a_repository = world.project.parent().unwrap().join("plain");
    fs::create_dir_all(&not_a_repository).unwrap();

    let refused = world.host.list(&not_a_repository).await.unwrap_err();

    assert!(refused.message().contains("failed"), "{}", refused.message());
}

#[tokio::test]
async fn reclaiming_removes_a_clean_orphan_and_reports_one_with_unsaved_changes() {
    let world = world();
    let clean = create(&world, "clean").await;
    let dirty = create(&world, "dirty").await;
    let active = create(&world, "active").await;
    fs::write(dirty.folder.join("half-done.txt"), "wip\n").unwrap();
    let reclaim = OrphanReclaimService::new(Arc::new(world.host.clone()));

    let report = reclaim.reclaim_orphans(&world.project, &HashSet::from([active.branch.clone()])).await.unwrap();

    assert_eq!(report.removed, vec![clean.clone()]);
    assert_eq!(report.kept_with_changes, vec![dirty.clone()]);
    assert!(!clean.folder.exists() && dirty.folder.exists() && active.folder.exists());
}
