use rekall_kernel::Instant;
use rekall_store::{company, project, task, ProjectStatus};
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};

use super::*;
use crate::worktree::ScriptedWorktreeHost;
use crate::EventBus;

struct World {
    service: TaskWorktreeService,
    host: Arc<ScriptedWorktreeHost>,
    db: sea_orm::DatabaseConnection,
    root: tempfile::TempDir,
    project_folder: PathBuf,
    project_id: Id,
}

async fn world() -> World {
    let database = rekall_store::open_in_memory().await.unwrap();
    let now = Instant::now();
    let root = tempfile::tempdir().unwrap();
    let company = company::Model { id: Id::random(), name: "Acme".into(), description: None, created_at: now, updated_at: now };
    company.clone().into_active_model().insert(&database.conn).await.unwrap();
    let project_folder = PathBuf::from("/work/vega");
    let project = project::Model {
        id: Id::random(),
        label: "vega".into(),
        title: "Vega".into(),
        status: ProjectStatus::Active,
        icon: "folder".into(),
        description: None,
        blueprint_markdown: None,
        repo_folder: Some(project_folder.to_string_lossy().into_owned()),
        auto_commit: false, main_branch: None,
        company_id: company.id,
        created_at: now,
        updated_at: now,
    };
    project.clone().into_active_model().insert(&database.conn).await.unwrap();
    let host = Arc::new(ScriptedWorktreeHost::new());
    let layout = WorktreeLayout::new(root.path().join("worktrees"));
    let service = TaskWorktreeService::new(Ctx::new(database.conn.clone(), EventBus::new()), host.clone(), layout);
    World { service, host, db: database.conn.clone(), root, project_folder, project_id: project.id }
}

async fn task_named(world: &World, label: &str) -> Id {
    let task = task::Model::new(label.into(), label.into(), world.project_id, Instant::now());
    task.clone().into_active_model().insert(&world.db).await.unwrap();
    task.id
}

async fn folder_of(world: &World, task_id: Id) -> Option<String> {
    task::Entity::find_by_id(task_id).one(&world.db).await.unwrap().unwrap().worktree_folder
}

#[tokio::test]
async fn a_task_gets_a_worktree_on_its_own_branch_and_keeps_it_for_the_next_run() {
    let world = world().await;
    let report = task_named(&world, "report").await;

    let first = world.service.ensure(report).await.unwrap();
    let again = world.service.ensure(report).await.unwrap();

    assert_eq!(first, world.root.path().join("worktrees").join("vega").join("report"));
    assert_eq!(again, first);
    assert_eq!(world.host.branches(), vec!["rekall/report".to_string()]);
    assert_eq!(folder_of(&world, report).await, Some(first.to_string_lossy().into_owned()));
}

#[tokio::test]
async fn a_worktree_git_would_not_make_leaves_the_task_without_one() {
    let world = world().await;
    let report = task_named(&world, "report").await;
    world.host.refuse("rekall/report");

    let refused = world.service.ensure(report).await.unwrap_err();

    assert!(matches!(refused, rekall_kernel::RekallError::IllegalArgument(_)));
    assert_eq!(folder_of(&world, report).await, None);
}

#[tokio::test]
async fn after_a_restart_a_task_forgets_a_folder_that_is_gone_and_a_clean_orphan_is_removed() {
    let world = world().await;
    let (kept, gone) = (task_named(&world, "kept").await, task_named(&world, "gone").await);
    world.service.ensure(kept).await.unwrap();
    let vanished = world.service.ensure(gone).await.unwrap();
    std::fs::remove_dir_all(&vanished).unwrap();
    world.host.create(&world.project_folder, "rekall/orphan", &world.root.path().join("worktrees/vega/orphan"), "HEAD").await.unwrap();

    world.service.reconcile().await.unwrap();

    assert_eq!(folder_of(&world, gone).await, None);
    assert!(folder_of(&world, kept).await.is_some());
    assert!(world.host.branches().contains(&"rekall/kept".to_string()));
    assert!(!world.host.branches().contains(&"rekall/orphan".to_string()));
}

#[tokio::test]
async fn an_orphan_with_unsaved_changes_is_left_alone() {
    let world = world().await;
    world.host.create(&world.project_folder, "rekall/orphan", &world.root.path().join("worktrees/vega/orphan"), "HEAD").await.unwrap();
    world.host.leave_unsaved_changes("rekall/orphan");

    world.service.reconcile().await.unwrap();

    assert_eq!(world.host.branches(), vec!["rekall/orphan".to_string()]);
}

#[tokio::test]
async fn releasing_a_clean_worktree_removes_the_folder_and_the_task_forgets_it() {
    let world = world().await;
    let report = task_named(&world, "report").await;
    let folder = world.service.ensure(report).await.unwrap();

    let released = world.service.release(report).await.unwrap();

    assert_eq!(released, Release::Removed);
    assert!(!folder.exists());
    assert_eq!(folder_of(&world, report).await, None);
}

#[tokio::test]
async fn releasing_a_worktree_with_unsaved_changes_leaves_it_as_it_was() {
    let world = world().await;
    let report = task_named(&world, "report").await;
    let folder = world.service.ensure(report).await.unwrap();
    world.host.leave_unsaved_changes("rekall/report");

    let released = world.service.release(report).await.unwrap();

    assert_eq!(released, Release::KeptWithChanges);
    assert!(folder.exists());
    assert_eq!(folder_of(&world, report).await, Some(folder.to_string_lossy().into_owned()));
}

#[tokio::test]
async fn a_task_that_never_had_a_worktree_has_nothing_to_release() {
    let world = world().await;
    let report = task_named(&world, "report").await;

    assert_eq!(world.service.release(report).await.unwrap(), Release::NoWorktree);
}

#[tokio::test]
async fn the_branch_is_reviewed_whether_or_not_its_worktree_is_still_there() {
    let world = world().await;
    let report = task_named(&world, "report").await;
    world.service.ensure(report).await.unwrap();

    let with_worktree = world.service.review_branch(report).await.unwrap().unwrap();
    let nothing = world.service.review_branch(task_named(&world, "other").await).await.unwrap();

    assert_eq!(with_worktree.branch, "rekall/report");
    assert!(with_worktree.worktree_folder.is_some());
    assert!(nothing.is_none());
}

#[tokio::test]
async fn an_accepted_branch_with_commits_is_merged_into_the_project() {
    let world = world().await;
    let report = task_named(&world, "report").await;
    world.service.ensure(report).await.unwrap();
    world.host.leave_commits("rekall/report");

    let merge = world.service.merge_accepted(report).await.unwrap();

    assert_eq!(merge, Merge::Merged);
    assert_eq!(world.host.merged(), vec!["rekall/report".to_string()]);
}

#[tokio::test]
async fn a_branch_the_project_already_has_is_not_merged_again() {
    let world = world().await;
    let report = task_named(&world, "report").await;
    world.service.ensure(report).await.unwrap();

    let merge = world.service.merge_accepted(report).await.unwrap();

    assert_eq!(merge, Merge::AlreadyMerged);
    assert!(world.host.merged().is_empty());
}

#[tokio::test]
async fn a_branch_with_unsaved_work_in_its_worktree_is_not_merged() {
    let world = world().await;
    let report = task_named(&world, "report").await;
    world.service.ensure(report).await.unwrap();
    world.host.leave_commits("rekall/report");
    world.host.leave_unsaved_changes("rekall/report");

    let merge = world.service.merge_accepted(report).await.unwrap();

    assert_eq!(merge, Merge::KeptWithChanges);
    assert!(world.host.merged().is_empty());
}

#[tokio::test]
async fn a_branch_that_conflicts_with_the_project_is_refused_and_not_merged() {
    let world = world().await;
    let report = task_named(&world, "report").await;
    world.service.ensure(report).await.unwrap();
    world.host.leave_commits("rekall/report");
    world.host.conflict_on("rekall/report");

    let merge = world.service.merge_accepted(report).await.unwrap();

    assert!(matches!(merge, Merge::Conflicts(_)));
    assert!(world.host.merged().is_empty());
}

#[tokio::test]
async fn a_task_without_a_branch_has_nothing_to_merge() {
    let world = world().await;
    let report = task_named(&world, "report").await;

    assert_eq!(world.service.merge_accepted(report).await.unwrap(), Merge::NothingToMerge);
}

async fn name_main_branch(world: &World, branch: &str) {
    let before = project::Entity::find_by_id(world.project_id).one(&world.db).await.unwrap().unwrap();
    project::Model { main_branch: Some(branch.into()), ..before }.into_active_model().reset_all().update(&world.db).await.unwrap();
}

async fn make_task_manager(world: &World) {
    use rekall_store::{external_connection, project_connection, ConnectionKind};
    let now = Instant::now();
    let connection = external_connection::Model::new("gitlab".into(), "GitLab".into(), ConnectionKind::Gitlab, Some("https://gitlab.example.com".into()), now);
    connection.clone().into_active_model().insert(&world.db).await.unwrap();
    let mut binding = project_connection::Model::new(world.project_id, connection.id, "gitlab".into(), Some("acme/vega".into()), now);
    binding.task_manager = true;
    binding.into_active_model().insert(&world.db).await.unwrap();
}

#[tokio::test]
async fn a_branch_starts_from_the_projects_main_branch_and_is_merged_into_it() {
    let world = world().await;
    name_main_branch(&world, "develop").await;
    let report = task_named(&world, "report").await;
    world.service.ensure(report).await.unwrap();
    world.host.leave_commits("rekall/report");

    world.service.merge_accepted(report).await.unwrap();

    assert_eq!(world.host.bases(), ["develop", "develop"]);
}

#[tokio::test]
async fn without_a_main_branch_the_folders_current_branch_is_the_base() {
    let world = world().await;
    let report = task_named(&world, "report").await;

    world.service.ensure(report).await.unwrap();

    assert_eq!(world.host.bases(), ["HEAD"]);
}

#[tokio::test]
async fn a_project_keeps_its_tasks_on_branches_when_it_has_a_task_manager_and_a_folder() {
    let world = world().await;
    let report = task_named(&world, "report").await;
    assert!(!world.service.requires_branch(report).await.unwrap());

    make_task_manager(&world).await;

    assert!(world.service.requires_branch(report).await.unwrap());
}
