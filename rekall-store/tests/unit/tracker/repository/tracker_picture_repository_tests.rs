use super::*;
use crate::enums::ProjectStatus;
use crate::{company, project, task};
use rekall_kernel::Instant;
use sea_orm::{ActiveModelTrait, ModelTrait};

async fn a_task(db: &impl ConnectionTrait) -> task::Model {
    let now = Instant::now();
    let company = company::Model { id: Id::random(), name: "Acme".into(), description: None, created_at: now, updated_at: now };
    company.clone().into_active_model().insert(db).await.unwrap();
    let project = project::Model {
        id: Id::random(), label: "vega".into(), title: "Vega".into(), status: ProjectStatus::Active, icon: "folder".into(),
        description: None, blueprint_markdown: None, repo_folder: None, auto_commit: false, main_branch: None, pull_request_flow: false, task_key: String::new(), company_id: company.id,
        created_at: now, updated_at: now,
    };
    project.clone().into_active_model().insert(db).await.unwrap();
    let task = task::Model::new("report".into(), "Report".into(), project.id, now);
    task.clone().into_active_model().insert(db).await.unwrap();
    task
}

fn uploaded(task_id: Id, destination: &str, reference: &str) -> Model {
    Model { task_id, destination: destination.into(), picture_path: "acme/vega/veg-1/login.png".into(), remote_reference: reference.into(), created_at: Instant::now() }
}

#[tokio::test]
async fn an_upload_is_read_back_for_its_destination_alone() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let task = a_task(db).await;
    let first = uploaded(task.id, "https://github.com/acme/app", "https://github.com/user-attachments/assets/1");
    save(db, first.clone()).await.unwrap();

    assert_eq!(list_of_destination(db, task.id, "https://github.com/acme/app").await.unwrap(), vec![first]);
    assert_eq!(list_of_destination(db, task.id, "https://github.com/acme/other").await.unwrap(), Vec::new());
}

#[tokio::test]
async fn saving_the_same_picture_again_replaces_its_reference() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let task = a_task(db).await;
    save(db, uploaded(task.id, "repo", "first")).await.unwrap();

    save(db, uploaded(task.id, "repo", "second")).await.unwrap();

    let kept = list_of_destination(db, task.id, "repo").await.unwrap();
    assert_eq!(kept.iter().map(|picture| picture.remote_reference.as_str()).collect::<Vec<_>>(), ["second"]);
}

#[tokio::test]
async fn deleting_a_task_forgets_its_uploads() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let task = a_task(db).await;
    save(db, uploaded(task.id, "repo", "first")).await.unwrap();

    task.clone().delete(db).await.unwrap();

    assert_eq!(list_of_destination(db, task.id, "repo").await.unwrap(), Vec::new());
}

#[tokio::test]
async fn a_reference_longer_than_the_column_is_refused() {
    let database = crate::open_temporary().await.unwrap();
    let task = a_task(&database.conn).await;
    let too_long = uploaded(task.id, "repo", &"x".repeat(crate::tracker_picture::REMOTE_REFERENCE_MAX_CHARACTERS + 1));

    assert!(save(&database.conn, too_long).await.is_err());
}
