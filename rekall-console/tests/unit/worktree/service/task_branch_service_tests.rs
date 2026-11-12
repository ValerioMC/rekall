use std::path::PathBuf;
use std::sync::Arc;

use rekall_core::worktree::{ScriptedWorktreeHost, WorktreeLayout};
use rekall_core::{Ctx, EventBus};
use rekall_kernel::Instant;
use rekall_store::{company, project, task, ProjectStatus};
use sea_orm::{ActiveModelTrait, IntoActiveModel};

use super::*;

struct World {
    service: TaskBranchService,
    worktrees: TaskWorktreeService,
    task_id: Id,
    _root: tempfile::TempDir,
}

async fn world() -> World {
    let database = rekall_store::open_in_memory().await.unwrap();
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
        repo_folder: Some(PathBuf::from("/work/vega").to_string_lossy().into_owned()),
        auto_commit: false, main_branch: None, pull_request_flow: false,
        company_id: company.id,
        created_at: now,
        updated_at: now,
    };
    project.clone().into_active_model().insert(&database.conn).await.unwrap();
    let task = task::Model::new("report".into(), "Report".into(), project.id, now);
    task.clone().into_active_model().insert(&database.conn).await.unwrap();
    let root = tempfile::tempdir().unwrap();
    let host = Arc::new(ScriptedWorktreeHost::new());
    let worktrees = TaskWorktreeService::new(Ctx::new(database.conn.clone(), EventBus::new()), host, WorktreeLayout::new(root.path().join("worktrees")));
    World { service: TaskBranchService::new(worktrees.clone()), worktrees, task_id: task.id, _root: root }
}

#[tokio::test]
async fn a_task_that_never_had_a_branch_shows_nothing() {
    let world = world().await;

    assert_eq!(world.service.view(world.task_id).await.unwrap(), None);
}

#[tokio::test]
async fn the_branch_is_shown_with_the_worktree_it_works_in() {
    let world = world().await;
    world.worktrees.ensure(world.task_id).await.unwrap();

    let view = world.service.view(world.task_id).await.unwrap().unwrap();

    assert_eq!(view.branch, "rekall/report");
    assert!(view.worktree_folder.is_some());
}
