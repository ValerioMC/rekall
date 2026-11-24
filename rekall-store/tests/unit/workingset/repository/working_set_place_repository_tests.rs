use super::*;
use crate::enums::ProjectStatus;
use crate::{company, project, task};
use rekall_kernel::Instant;
use sea_orm::IntoActiveModel;

async fn a_project_with_two_tasks(db: &impl ConnectionTrait) -> (project::Model, Vec<task::Model>) {
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
    for label in ["alpha", "beta"] {
        let task = task::Model::new(label.into(), label.to_uppercase(), project.id, now);
        task.clone().into_active_model().insert(db).await.unwrap();
        tasks.push(task);
    }
    (project, tasks)
}

#[tokio::test]
async fn the_place_remembered_for_a_project_is_replaced_not_added() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let (project, tasks) = a_project_with_two_tasks(db).await;

    remember(db, project.id, Some(tasks[0].id)).await.unwrap();
    remember(db, project.id, Some(tasks[1].id)).await.unwrap();

    let places = list(db).await.unwrap();
    assert_eq!(places, [Model { project_id: project.id, task_id: Some(tasks[1].id) }]);
}

#[tokio::test]
async fn the_task_list_is_remembered_as_no_task() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let (project, tasks) = a_project_with_two_tasks(db).await;
    remember(db, project.id, Some(tasks[0].id)).await.unwrap();

    remember(db, project.id, None).await.unwrap();

    assert_eq!(find_of_project(db, project.id).await.unwrap().unwrap().task_id, None);
}

#[tokio::test]
async fn a_project_that_does_not_exist_cannot_be_remembered() {
    let database = crate::open_temporary().await.unwrap();

    assert!(remember(&database.conn, Id::random(), None).await.is_err());
    assert!(list(&database.conn).await.unwrap().is_empty());
}

#[tokio::test]
async fn a_deleted_remembered_task_sends_the_project_back_to_its_task_list() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let (project, tasks) = a_project_with_two_tasks(db).await;
    remember(db, project.id, Some(tasks[0].id)).await.unwrap();

    task::Entity::delete_by_id(tasks[0].id).exec(db).await.unwrap();

    assert_eq!(find_of_project(db, project.id).await.unwrap().unwrap().task_id, None);
}
