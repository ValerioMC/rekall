use std::sync::Arc;

use rekall_kernel::Instant;
use rekall_store::{company, project, task, ProjectStatus};
use sea_orm::{ActiveModelTrait, IntoActiveModel};

use super::*;
use crate::connection::{ScriptedTaskTracker, Secret};
use crate::picture::InMemoryPictureStore;
use crate::EventBus;

const LOGIN: &str = "acme/app/app-1/login-3f9a1c2b.png";

struct World {
    service: TrackerPictureService,
    tracker: Arc<ScriptedTaskTracker>,
    task_id: Id,
}

async fn world() -> World {
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
    let task = task::Model::new("first".into(), "First".into(), project.id, now);
    task.clone().into_active_model().insert(&ctx.db).await.unwrap();
    let pictures = Arc::new(InMemoryPictureStore::new());
    pictures.put(&PicturePath::parse(LOGIN).unwrap(), b"picture");
    let tracker = Arc::new(ScriptedTaskTracker::new());
    World { service: TrackerPictureService::new(ctx, pictures, tracker.clone()), tracker, task_id: task.id }
}

fn endpoint(kind: ConnectionKind, location: &str) -> TrackerEndpoint {
    TrackerEndpoint { kind, site_url: "https://tracker.example.com".into(), proxy_url: None, location: location.into(), credential: Secret::new("pat:token") }
}

fn shown_twice() -> String {
    format!("![a](/api/images/{LOGIN}) ![b](/api/images/{LOGIN}#right)")
}

#[tokio::test]
async fn a_picture_shown_twice_is_uploaded_once() {
    let world = world().await;

    let sent = world.service.send(&endpoint(ConnectionKind::Gitlab, "acme/app"), world.task_id, None, &shown_twice()).await.unwrap();

    assert_eq!(world.tracker.uploaded().len(), 1);
    assert!(!sent.contains("/api/images/"));
}

#[tokio::test]
async fn another_repository_gets_its_own_upload() {
    let world = world().await;

    for location in ["acme/app", "acme/other"] {
        world.service.send(&endpoint(ConnectionKind::Github, location), world.task_id, None, &shown_twice()).await.unwrap();
    }

    assert_eq!(world.tracker.uploaded().len(), 2);
}

#[tokio::test]
async fn a_jira_attachment_belongs_to_its_issue() {
    let world = world().await;
    let jira = endpoint(ConnectionKind::Jira, "ENG");

    world.service.send(&jira, world.task_id, Some("ENG-1"), &shown_twice()).await.unwrap();
    world.service.send(&jira, world.task_id, Some("ENG-2"), &shown_twice()).await.unwrap();
    let without_issue = world.service.send(&jira, world.task_id, None, &shown_twice()).await;

    assert_eq!(world.tracker.uploaded().iter().map(|(issue, _)| issue.as_deref()).collect::<Vec<_>>(), [Some("ENG-1"), Some("ENG-2")]);
    assert!(without_issue.is_err());
}

#[tokio::test]
async fn a_text_without_pictures_is_sent_as_it_is() {
    let world = world().await;

    let sent = world.service.send(&endpoint(ConnectionKind::Jira, "ENG"), world.task_id, None, "Do it.").await.unwrap();

    assert_eq!(sent, "Do it.");
}

#[tokio::test]
async fn a_rejected_credential_is_named_as_such() {
    let world = world().await;
    world.tracker.refuse_uploads(TrackerFailure::Rejected { status: 401 });

    let refused = world.service.send(&endpoint(ConnectionKind::Gitlab, "acme/app"), world.task_id, None, &shown_twice()).await.unwrap_err();

    assert!(refused.message().contains("GitLab rejected the stored credential (HTTP 401)"));
}
