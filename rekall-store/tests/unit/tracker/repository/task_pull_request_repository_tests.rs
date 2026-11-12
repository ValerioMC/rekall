use super::*;
use crate::enums::ProjectStatus;
use crate::{company, project, task};
use rekall_kernel::Instant;
use sea_orm::ActiveModelTrait;

async fn a_task(db: &impl ConnectionTrait) -> task::Model {
    let now = Instant::now();
    let company = company::Model { id: Id::random(), name: "Acme".into(), description: None, created_at: now, updated_at: now };
    company.clone().into_active_model().insert(db).await.unwrap();
    let project = project::Model {
        id: Id::random(), label: "vega".into(), title: "Vega".into(), status: ProjectStatus::Active, icon: "folder".into(),
        description: None, blueprint_markdown: None, repo_folder: None, auto_commit: false, main_branch: None, pull_request_flow: true, company_id: company.id,
        created_at: now, updated_at: now,
    };
    project.clone().into_active_model().insert(db).await.unwrap();
    let task = task::Model::new("report".into(), "Report".into(), project.id, now);
    task.clone().into_active_model().insert(db).await.unwrap();
    task
}

fn opened(task_id: Id, number: i64) -> Model {
    let now = Instant::now();
    Model { task_id, number: Some(number), url: Some(format!("https://github.com/acme/app/pull/{number}")), failure: None, created_at: now, updated_at: now }
}

#[tokio::test]
async fn a_task_that_never_reached_a_claim_has_no_pull_request() {
    let database = crate::open_temporary().await.unwrap();
    let task = a_task(&database.conn).await;

    assert_eq!(find_of_task(&database.conn, task.id).await.unwrap(), None);
}

#[tokio::test]
async fn a_saved_pull_request_is_read_back_and_a_second_save_replaces_it() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let task = a_task(db).await;
    let first = opened(task.id, 5);
    save(db, first.clone()).await.unwrap();
    assert_eq!(find_of_task(db, task.id).await.unwrap(), Some(first.clone()));

    let failed = Model { number: None, url: None, failure: Some("The push was refused.".into()), ..first };
    save(db, failed.clone()).await.unwrap();

    assert_eq!(list_of_tasks(db, vec![task.id]).await.unwrap(), vec![failed]);
}

#[tokio::test]
async fn a_failure_longer_than_the_column_is_refused() {
    let database = crate::open_temporary().await.unwrap();
    let task = a_task(&database.conn).await;
    let too_long = Model { failure: Some("x".repeat(crate::task_pull_request::FAILURE_MAX_CHARACTERS + 1)), ..opened(task.id, 5) };

    assert!(save(&database.conn, too_long).await.is_err());
}
