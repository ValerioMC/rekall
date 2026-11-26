use std::sync::Arc;

use rekall_kernel::{Id, Instant};
use rekall_store::{company, external_connection, project, project_connection, task, task_tracker_link, ConnectionKind, ProjectStatus, TaskKind};
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};

use super::*;
use crate::connection::{InMemorySecretStore, ScriptedTaskTracker, TrackerFailure};
use crate::picture::{InMemoryPictureStore, PicturePath};
use crate::EventBus;

struct World {
    service: TaskTrackerService,
    tracker: Arc<ScriptedTaskTracker>,
    pictures: Arc<InMemoryPictureStore>,
    ctx: Ctx,
    project_id: Id,
}

async fn world(kind: ConnectionKind, target: Option<&str>, task_manager: bool) -> World {
    let database = rekall_store::open_in_memory().await.unwrap();
    let ctx = Ctx::new(database.conn.clone(), EventBus::new());
    let now = Instant::now();
    let company = company::Model { id: Id::random(), name: "Acme".into(), description: None, created_at: now, updated_at: now };
    company.clone().into_active_model().insert(&ctx.db).await.unwrap();
    let project = project::Model {
        id: Id::random(), label: "app".into(), title: "App".into(), status: ProjectStatus::Active, icon: "folder".into(), description: None,
        blueprint_markdown: None, repo_folder: None, auto_commit: false, main_branch: None, pull_request_flow: false, task_key: String::new(), company_id: company.id, created_at: now, updated_at: now,
    };
    project.clone().into_active_model().insert(&ctx.db).await.unwrap();
    let mut connection = external_connection::Model::new("tracker".into(), "Tracker".into(), kind, Some("https://tracker.example.com".into()), now);
    connection.secret_set_at = Some(now);
    connection.clone().into_active_model().insert(&ctx.db).await.unwrap();
    let mut binding = project_connection::Model::new(project.id, connection.id, "tracker".into(), target.map(str::to_string), now);
    binding.task_manager = task_manager;
    binding.clone().into_active_model().insert(&ctx.db).await.unwrap();
    let secrets = Arc::new(InMemorySecretStore::new());
    secrets.put(&connection.secret_ref, &Secret::new("token")).unwrap();
    let tracker = Arc::new(ScriptedTaskTracker::new());
    let pictures = Arc::new(InMemoryPictureStore::new());
    let uploads = TrackerPictureService::new(ctx.clone(), pictures.clone(), tracker.clone());
    World { service: TaskTrackerService::new(ctx.clone(), secrets, tracker.clone(), uploads), tracker, pictures, ctx, project_id: project.id }
}

impl World {
    async fn task(&self, label: &str, edit: impl FnOnce(&mut task::Model)) -> Id {
        let mut task = task::Model::new(label.into(), format!("Task {label}"), self.project_id, Instant::now());
        task.description = Some("Do it.".into());
        edit(&mut task);
        task.clone().into_active_model().insert(&self.ctx.db).await.unwrap();
        task.id
    }

    async fn link(&self, id: Id) -> Option<task_tracker_link::Model> {
        rekall_store::repository::task_tracker_link::find_of_task(&self.ctx.db, id).await.unwrap()
    }

    async fn link_issue(&self, id: Id, key: &str) {
        let now = Instant::now();
        let link = task_tracker_link::Model { task_id: id, issue_key: Some(key.into()), issue_url: None, sync_error: None, synced_fingerprint: None, created_at: now, updated_at: now };
        rekall_store::repository::task_tracker_link::save(&self.ctx.db, link).await.unwrap();
    }
}

#[tokio::test]
async fn a_task_without_an_issue_creates_one_with_its_title_alone_and_keeps_its_link() {
    let world = world(ConnectionKind::Gitlab, Some("acme/app"), true).await;
    let id = world.task("first", |_| {}).await;

    world.service.create_missing(id).await.unwrap();

    let created = world.tracker.created();
    assert_eq!(created.len(), 1);
    assert_eq!(created[0].0, "acme/app");
    assert_eq!(created[0].1.title, "Task first");
    assert_eq!(created[0].1.description, "");
    let link = world.link(id).await.unwrap();
    assert_eq!(link.issue_key.as_deref(), Some("acme/app-1"));
    assert_eq!(link.issue_url.as_deref(), Some("https://tracker.example.com/issues/acme/app-1"));
    assert_eq!(link.sync_error, None);
}

#[tokio::test]
async fn a_task_that_has_an_issue_is_not_written_again_by_its_creation() {
    let world = world(ConnectionKind::Github, Some("acme/app"), true).await;
    let id = world.task("first", |_| {}).await;
    world.link_issue(id, "12").await;

    world.service.create_missing(id).await.unwrap();

    assert!(world.tracker.created().is_empty());
    assert!(world.tracker.updated().is_empty());
}

#[tokio::test]
async fn syncing_a_task_without_an_issue_creates_it_with_its_description() {
    let world = world(ConnectionKind::Gitlab, Some("acme/app"), true).await;
    let id = world.task("first", |_| {}).await;

    world.service.sync_content(id).await.unwrap();

    assert_eq!(world.tracker.created()[0].1.description, "Do it.");
    assert!(world.link(id).await.unwrap().issue_key.is_some());
}

#[tokio::test]
async fn syncing_a_task_that_has_an_issue_rewrites_it_instead_of_creating_another() {
    let world = world(ConnectionKind::Github, Some("acme/app"), true).await;
    let id = world.task("first", |_| {}).await;
    world.link_issue(id, "12").await;

    world.service.sync_content(id).await.unwrap();

    assert!(world.tracker.created().is_empty());
    let updated = world.tracker.updated();
    assert_eq!(updated.len(), 1);
    assert_eq!(updated[0].0, "12");
    assert_eq!(updated[0].1.description, "Do it.");
}

#[tokio::test]
async fn a_sync_keeps_the_fingerprint_of_what_it_wrote() {
    let world = world(ConnectionKind::Github, Some("acme/app"), true).await;
    let id = world.task("first", |_| {}).await;
    world.link_issue(id, "12").await;
    let written = TrackerIssue::of(&task::Entity::find_by_id(id).one(&world.ctx.db).await.unwrap().unwrap());

    world.service.sync_content(id).await.unwrap();

    assert_eq!(world.link(id).await.unwrap().synced_fingerprint, Some(written.fingerprint()));
}

#[tokio::test]
async fn a_jira_project_key_is_written_upper_case() {
    let world = world(ConnectionKind::Jira, Some("eng"), true).await;
    let id = world.task("first", |_| {}).await;

    world.service.create_missing(id).await.unwrap();

    assert_eq!(world.tracker.created()[0].0, "ENG");
}

#[tokio::test]
async fn a_project_without_a_task_manager_creates_nothing_and_refuses_a_sync() {
    let world = world(ConnectionKind::Gitlab, Some("acme/app"), false).await;
    let id = world.task("first", |_| {}).await;

    world.service.create_missing(id).await.unwrap();
    let refused = world.service.sync_content(id).await;

    assert!(refused.is_err());
    assert!(world.tracker.created().is_empty());
    assert!(world.link(id).await.is_none());
}

#[tokio::test]
async fn an_epic_and_a_quick_task_are_not_created() {
    let world = world(ConnectionKind::Gitlab, Some("acme/app"), true).await;
    let epic = world.task("epic", |task| task.kind = TaskKind::Epic).await;
    let quick = world.task("quick", |task| task.quick = true).await;

    world.service.create_missing(epic).await.unwrap();
    world.service.create_missing(quick).await.unwrap();

    assert!(world.tracker.created().is_empty());
}

#[tokio::test]
async fn a_refusal_is_kept_on_the_task_and_returned_and_cleared_by_the_next_sync_that_works() {
    let world = world(ConnectionKind::Gitlab, Some("acme/app"), true).await;
    let id = world.task("first", |_| {}).await;
    world.tracker.fail_with(TrackerFailure::Rejected { status: 401 });

    let refused = world.service.sync_content(id).await;

    assert!(refused.unwrap_err().message().contains("rejected the stored credential (HTTP 401)"));
    let failed = world.link(id).await.unwrap();
    assert_eq!(failed.issue_key, None);
    assert!(failed.sync_error.unwrap().contains("rejected the stored credential (HTTP 401)"));

    world.tracker.recover();
    world.service.sync_content(id).await.unwrap();

    let recovered = world.link(id).await.unwrap();
    assert!(recovered.issue_key.is_some());
    assert_eq!(recovered.sync_error, None);
}

#[tokio::test]
async fn a_binding_without_a_target_is_kept_as_a_failure() {
    let world = world(ConnectionKind::Gitlab, None, true).await;
    let id = world.task("first", |_| {}).await;

    assert!(world.service.create_missing(id).await.is_err());

    assert!(world.tracker.created().is_empty());
    assert!(world.link(id).await.unwrap().sync_error.unwrap().contains("target"));
}

const LOGIN: &str = "acme/app/app-1/login-3f9a1c2b.png";
const MOCK: &str = "acme/app/app-1/mock-77aa01de.png";

impl World {
    fn picture(&self, path: &str) -> String {
        self.pictures.put(&PicturePath::parse(path).unwrap(), b"picture");
        format!("![Shown](/api/images/{path}#right)")
    }

    async fn describe(&self, id: Id, description: String) {
        let mut task = task::Entity::find_by_id(id).one(&self.ctx.db).await.unwrap().unwrap().into_active_model();
        task.description = sea_orm::Set(Some(description));
        task.update(&self.ctx.db).await.unwrap();
    }
}

#[tokio::test]
async fn the_first_push_creates_the_issue_with_descriptions_then_uploads_and_shows_the_copies() {
    let world = world(ConnectionKind::Gitlab, Some("acme/app"), true).await;
    let id = world.task("first", |_| {}).await;
    world.describe(id, format!("Build {}", world.picture(LOGIN))).await;

    world.service.sync_content(id).await.unwrap();

    assert_eq!(world.tracker.created()[0].1.description, "Build Shown");
    assert_eq!(world.tracker.uploaded(), [(Some("acme/app-1".to_string()), LOGIN.to_string())]);
    assert_eq!(world.tracker.updated()[0].1.description, "Build ![Shown](https://tracker.example.com/uploads/login-3f9a1c2b.png)");
    let written = TrackerIssue::of(&task::Entity::find_by_id(id).one(&world.ctx.db).await.unwrap().unwrap());
    assert_eq!(world.link(id).await.unwrap().synced_fingerprint, Some(written.fingerprint()));
}

#[tokio::test]
async fn a_second_push_uploads_nothing_and_a_new_picture_only_is_uploaded() {
    let world = world(ConnectionKind::Github, Some("acme/app"), true).await;
    let id = world.task("first", |_| {}).await;
    let login = world.picture(LOGIN);
    world.describe(id, login.clone()).await;
    world.service.sync_content(id).await.unwrap();

    world.service.sync_content(id).await.unwrap();
    assert_eq!(world.tracker.uploaded().len(), 1);

    world.describe(id, format!("{login} {}", world.picture(MOCK))).await;
    world.service.sync_content(id).await.unwrap();

    assert_eq!(world.tracker.uploaded().iter().map(|(_, path)| path.as_str()).collect::<Vec<_>>(), [LOGIN, MOCK]);
    let description = world.tracker.updated().last().unwrap().1.description.clone();
    assert!(description.contains("uploads/login-3f9a1c2b.png") && description.contains("uploads/mock-77aa01de.png"));
}

#[tokio::test]
async fn a_refused_upload_keeps_the_issue_description_and_the_reason_on_the_task() {
    let world = world(ConnectionKind::Github, Some("acme/app"), true).await;
    let id = world.task("first", |_| {}).await;
    world.link_issue(id, "12").await;
    world.describe(id, world.picture(LOGIN)).await;
    world.tracker.refuse_uploads(TrackerFailure::Failed { reason: "it answered HTTP 404".into() });

    let refused = world.service.sync_content(id).await.unwrap_err();

    assert!(refused.message().contains("GitHub could not take the picture login-3f9a1c2b.png: it answered HTTP 404"));
    assert!(world.tracker.updated().is_empty());
    let link = world.link(id).await.unwrap();
    assert_eq!(link.sync_error.as_deref(), Some(refused.message()));
    assert_eq!(link.synced_fingerprint, None);
}

#[tokio::test]
async fn an_upload_refused_after_the_issue_was_created_keeps_its_key_for_the_next_push() {
    let world = world(ConnectionKind::Gitlab, Some("acme/app"), true).await;
    let id = world.task("first", |_| {}).await;
    world.describe(id, world.picture(LOGIN)).await;
    world.tracker.refuse_uploads(TrackerFailure::Failed { reason: "too large".into() });

    assert!(world.service.sync_content(id).await.is_err());
    world.tracker.recover();
    world.service.sync_content(id).await.unwrap();

    assert_eq!(world.tracker.created().len(), 1);
    assert_eq!(world.tracker.updated().len(), 1);
    assert_eq!(world.link(id).await.unwrap().sync_error, None);
}

#[tokio::test]
async fn a_picture_missing_from_disk_is_written_as_its_description() {
    let world = world(ConnectionKind::Github, Some("acme/app"), true).await;
    let id = world.task("first", |_| {}).await;
    world.link_issue(id, "12").await;
    world.describe(id, format!("Build ![Gone](/api/images/{LOGIN})")).await;

    world.service.sync_content(id).await.unwrap();

    assert!(world.tracker.uploaded().is_empty());
    assert_eq!(world.tracker.updated()[0].1.description, "Build Gone");
}
