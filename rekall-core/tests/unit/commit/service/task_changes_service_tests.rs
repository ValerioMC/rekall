use super::*;
use crate::commit::model::task_changes_snapshots::{MAX_SNAPSHOTS, SNAPSHOT_TTL};
use crate::commit::{ChangeSourceId, run_git};
use crate::EventBus;
use rekall_kernel::Instant as LoggedAt;
use rekall_store::{company, project, project_component, task, task_step, ProjectStatus};
use sea_orm::{ActiveModelTrait, IntoActiveModel};
use std::time::Duration;

#[test]
fn a_patch_within_the_cap_is_kept_whole() {
    let (kept, truncated) = capped("+short\n".to_string());

    assert_eq!(kept, "+short\n");
    assert!(!truncated);
}

#[test]
fn a_patch_past_the_cap_is_cut_on_a_character_and_flagged() {
    let (kept, truncated) = capped("é".repeat(PATCH_MAX + 10));

    assert_eq!(kept.chars().count(), PATCH_MAX);
    assert!(truncated);
}

#[tokio::test]
async fn batches_resolve_the_saved_working_tree_and_every_matching_commit_in_scope() {
    let world = ChangesWorld::new().await;
    let index = world.service.index(world.task_id).await.unwrap();
    let primary = &index.repositories[0];
    assert_eq!(index.repositories.len(), 2);
    assert_eq!(
        primary.commits.iter().map(|commit| commit.reference_id).collect::<Vec<_>>(),
        world.primary_references,
    );
    let unavailable = primary.commits.iter().find(|commit| !commit.available).unwrap();
    assert!(unavailable.files.is_empty());
    let working_tree = primary.working_tree.as_ref().unwrap();
    assert_eq!(working_tree.files[0].additions, 1);
    std::fs::write(world.checkout.path().join("shared.txt"), "written after the index\n").unwrap();

    let project_file = ChangeFileKey { component_id: None, path: "shared.txt".into() };
    let component_file = ChangeFileKey { component_id: Some(world.component_id), path: project_file.path.clone() };
    let batch = world.service.hunks(world.task_id, index.snapshot_id, TaskChangesScope::All, &[project_file.clone(), component_file.clone()]).await.unwrap();
    let contents: Vec<_> = batch.iter().map(|entry| (&entry.file, entry.source, entry.hunks[0].lines.last().unwrap().text.as_str())).collect();
    assert_eq!(contents, [
        (&project_file, ChangeSourceId::Commit { reference_id: world.primary_commit }, "project commit"),
        (&project_file, ChangeSourceId::WorkingTree, "working before the index"),
        (&component_file, ChangeSourceId::Commit { reference_id: world.component_commit }, "component commit"),
    ]);

    let scopes = [
        (TaskChangesScope::Step { step_id: world.step_id }, &project_file, batch[0].source),
        (TaskChangesScope::Commit { reference_id: world.primary_commit }, &project_file, batch[0].source),
        (TaskChangesScope::WorkingTree { component_id: None }, &project_file, batch[1].source),
        (TaskChangesScope::Repository { component_id: Some(world.component_id) }, &component_file, batch[2].source),
    ];
    for (scope, file, source) in scopes {
        let hunks = world.service.hunks(world.task_id, index.snapshot_id, scope, std::slice::from_ref(file)).await.unwrap();
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].source, source);
    }
    let current = world.service.index(world.task_id).await.unwrap();
    let current_tree = world.service.hunks(world.task_id, current.snapshot_id, TaskChangesScope::WorkingTree { component_id: None }, &[project_file]).await.unwrap();
    assert_eq!(current_tree[0].hunks[0].lines.last().unwrap().text, "written after the index");
}

#[test]
fn snapshots_expire_at_their_deadline_and_are_bound_to_the_task() {
    let mut snapshots = TaskChangesSnapshots::default();
    let now = Instant::now();
    let task = Id::random();
    let snapshot = TaskChangesSnapshot::new(task, Vec::new());
    let id = snapshot.id;
    snapshots.insert(now, snapshot);

    assert!(matches!(snapshots.require(now, Id::random(), id), Err(RekallError::NotFound(_))));
    assert!(snapshots.require(now + SNAPSHOT_TTL - Duration::from_secs(1), task, id).is_ok());
    assert!(matches!(snapshots.require(now + SNAPSHOT_TTL, task, id), Err(RekallError::Conflict(_))));
}

#[test]
fn retaining_another_snapshot_at_capacity_releases_the_oldest() {
    let mut snapshots = TaskChangesSnapshots::default();
    let now = Instant::now();
    let task = Id::random();
    let snapshot = TaskChangesSnapshot::new(task, Vec::new());
    let oldest = snapshot.id;
    snapshots.insert(now, snapshot);

    for _ in 0..MAX_SNAPSHOTS {
        snapshots.insert(now, TaskChangesSnapshot::new(task, Vec::new()));
    }

    assert!(matches!(snapshots.require(now, task, oldest), Err(RekallError::Conflict(_))));
}

struct ChangesWorld {
    service: TaskChangesService,
    checkout: tempfile::TempDir,
    task_id: Id,
    step_id: Id,
    component_id: Id,
    primary_commit: Id,
    primary_references: [Id; 2],
    component_commit: Id,
}

impl ChangesWorld {
    async fn new() -> Self {
        let checkout = checkout_with_pending_change().await;
        let database = rekall_store::open_in_memory().await.unwrap();
        let now = LoggedAt::now();
        let company = company::Model {
            id: Id::random(), name: "Acme".into(), description: None, created_at: now, updated_at: now,
        }.into_active_model().insert(&database.conn).await.unwrap();
        let project = project::Model {
            id: Id::random(), label: "app".into(), title: "App".into(), status: ProjectStatus::Active,
            icon: "folder".into(), description: None, blueprint_markdown: None,
            repo_folder: Some(checkout.path().to_string_lossy().into_owned()), auto_commit: false, main_branch: None,
            pull_request_flow: false, task_key: "APP".into(), company_id: company.id, created_at: now, updated_at: now,
        }.into_active_model().insert(&database.conn).await.unwrap();
        let component = add_component(&database.conn, project.id, "lib", 0, now).await;
        add_component(&database.conn, project.id, "untouched", 1, now).await;
        let task = task::Model::new("diffs".into(), "Diffs".into(), project.id, now)
            .into_active_model().insert(&database.conn).await.unwrap();
        let step = task_step::Model::new(task.id, "Index diffs".into(), 0, now)
            .into_active_model().insert(&database.conn).await.unwrap();
        let primary = reference(task.id, Some(step.id), None, Some("project commit"), now);
        let missing = reference(task.id, None, None, None, now);
        let other = reference(task.id, None, Some(component.id), Some("component commit"), now);
        let primary_commit = primary.id;
        let component_commit = other.id;
        let mut tied_references = [primary, missing];
        tied_references.sort_by_key(|reference| reference.id);
        let primary_references = tied_references.each_ref().map(|reference| reference.id);
        for reference in tied_references.into_iter().rev().chain([other]) {
            reference.into_active_model().insert(&database.conn).await.unwrap();
        }
        let service = TaskChangesService::new(Ctx::new(database.conn, EventBus::new()));
        Self {
            service, checkout, task_id: task.id, step_id: step.id, component_id: component.id,
            primary_commit, primary_references, component_commit,
        }
    }
}

async fn add_component(
    db: &sea_orm::DatabaseConnection,
    project_id: Id,
    name: &str,
    position: i32,
    now: LoggedAt,
) -> project_component::Model {
    project_component::Model {
        id: Id::random(), project_id, name: name.into(), folder: format!("/missing/{project_id}/{name}"),
        summary: String::new(), description: String::new(), position, created_at: now, updated_at: now,
    }.into_active_model().insert(db).await.unwrap()
}

fn reference(task_id: Id, step_id: Option<Id>, component_id: Option<Id>, text: Option<&str>, now: LoggedAt) -> commit_reference::Model {
    commit_reference::Model {
        id: Id::random(), task_id, step_id, component_id, commit_hash: "0".repeat(40), comment: "Logged commit".into(),
        diff: text.map(|text| format!("diff --git a/shared.txt b/shared.txt\n+++ b/shared.txt\n@@ -0,0 +1 @@\n+{text}\n")),
        in_context: false, created_at: now,
    }
}

async fn checkout_with_pending_change() -> tempfile::TempDir {
    let checkout = tempfile::tempdir().unwrap();
    git(checkout.path(), &["init", "-q"]).await;
    std::fs::write(checkout.path().join("shared.txt"), "base\n").unwrap();
    git(checkout.path(), &["add", "."]).await;
    git(checkout.path(), &["-c", "user.name=Test", "-c", "user.email=test@example.com", "commit", "-qm", "base"]).await;
    std::fs::write(checkout.path().join("shared.txt"), "working before the index\n").unwrap();
    checkout
}

async fn git(folder: &Path, args: &[&str]) {
    let result = run_git(folder, args, None, Duration::from_secs(5)).await.unwrap();
    assert!(result.ok(), "git {args:?}: {}", result.stderr);
}
