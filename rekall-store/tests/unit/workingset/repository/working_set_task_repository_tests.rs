use super::*;
use crate::enums::ProjectStatus;
use crate::{company, project, task};
use rekall_kernel::Instant;
use sea_orm::{ActiveModelTrait, IntoActiveModel};

async fn tasks_in_one_project(db: &impl ConnectionTrait, labels: &[&str]) -> Vec<task::Model> {
    let now = Instant::now();
    let company = company::Model { id: Id::random(), name: "Acme".into(), description: None, created_at: now, updated_at: now };
    company.clone().into_active_model().insert(db).await.unwrap();
    let project = project::Model {
        id: Id::random(), label: "vega".into(), title: "Vega".into(), status: ProjectStatus::Active, icon: "folder".into(),
        description: None, blueprint_markdown: None, repo_folder: None, auto_commit: false, main_branch: None, pull_request_flow: false, task_key: String::new(), company_id: company.id,
        created_at: now, updated_at: now,
    };
    project.clone().into_active_model().insert(db).await.unwrap();
    let mut tasks = Vec::new();
    for label in labels {
        let task = task::Model::new((*label).into(), label.to_uppercase(), project.id, now);
        task.clone().into_active_model().insert(db).await.unwrap();
        tasks.push(task);
    }
    tasks
}

async fn open_ids(db: &impl ConnectionTrait) -> Vec<Id> {
    list(db).await.unwrap().into_iter().map(|open| open.task_id).collect()
}

#[tokio::test]
async fn tasks_open_in_the_order_they_are_opened_and_opening_again_changes_nothing() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let tasks = tasks_in_one_project(db, &["alpha", "beta", "gamma"]).await;

    open(db, tasks[1].id).await.unwrap();
    open(db, tasks[0].id).await.unwrap();
    open(db, tasks[1].id).await.unwrap();

    assert_eq!(open_ids(db).await, [tasks[1].id, tasks[0].id]);
    assert_eq!(find_by_task(db, tasks[1].id).await.unwrap().unwrap().position, 0);
}

#[tokio::test]
async fn opening_a_task_that_does_not_exist_is_not_found() {
    let database = crate::open_temporary().await.unwrap();

    let refused = open(&database.conn, Id::random()).await.unwrap_err();

    assert!(matches!(refused, RekallError::NotFound(_)), "{refused:?}");
    assert!(list(&database.conn).await.unwrap().is_empty());
}

#[tokio::test]
async fn closing_a_task_closes_the_gap_it_leaves() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let tasks = tasks_in_one_project(db, &["alpha", "beta", "gamma"]).await;
    for task in &tasks {
        open(db, task.id).await.unwrap();
    }

    close(db, tasks[0].id).await.unwrap();

    let remaining = list(db).await.unwrap();
    assert_eq!(remaining.iter().map(|open| (open.task_id, open.position)).collect::<Vec<_>>(), [(tasks[1].id, 0), (tasks[2].id, 1)]);
}

#[tokio::test]
async fn closing_a_task_that_is_not_open_changes_nothing() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let tasks = tasks_in_one_project(db, &["alpha", "beta"]).await;
    open(db, tasks[0].id).await.unwrap();

    close(db, tasks[1].id).await.unwrap();

    assert_eq!(open_ids(db).await, [tasks[0].id]);
}

#[tokio::test]
async fn a_deleted_task_leaves_the_set() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let tasks = tasks_in_one_project(db, &["alpha", "beta"]).await;
    open(db, tasks[0].id).await.unwrap();
    open(db, tasks[1].id).await.unwrap();

    task::Entity::delete_by_id(tasks[0].id).exec(db).await.unwrap();

    assert_eq!(open_ids(db).await, [tasks[1].id]);
}
