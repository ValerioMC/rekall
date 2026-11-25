use super::*;
use crate::enums::ProjectStatus;
use crate::{company, project, task, task_step};
use rekall_kernel::Instant;
use sea_orm::IntoActiveModel;

async fn a_task_with_a_step(db: &impl ConnectionTrait) -> (task::Model, task_step::Model) {
    let now = Instant::now();
    let company = company::Model { id: Id::random(), name: "Acme".into(), description: None, created_at: now, updated_at: now };
    company.clone().into_active_model().insert(db).await.unwrap();
    let project = project::Model {
        id: Id::random(), label: "vega".into(), title: "Vega".into(), status: ProjectStatus::Active, icon: "folder".into(),
        description: None, blueprint_markdown: None, repo_folder: None, auto_commit: false, main_branch: None, pull_request_flow: false, task_key: String::new(), company_id: company.id,
        created_at: now, updated_at: now,
    };
    project.clone().into_active_model().insert(db).await.unwrap();
    let task = task::Model::new("alpha".into(), "Alpha".into(), project.id, now);
    task.clone().into_active_model().insert(db).await.unwrap();
    let step = task_step::Model::new(task.id, "First".into(), 0, now);
    step.clone().into_active_model().insert(db).await.unwrap();
    (task, step)
}

fn fill(task_id: Id, step_id: Option<Id>, used_tokens: i64) -> Model {
    Model { id: Id::random(), task_id, step_id, used_tokens, window_tokens: 200_000, recorded_at: Instant::now() }
}

#[tokio::test]
async fn upserting_twice_leaves_one_row_with_the_last_value() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let (task, step) = a_task_with_a_step(db).await;

    upsert(db, fill(task.id, Some(step.id), 1_000)).await.unwrap();
    upsert(db, fill(task.id, Some(step.id), 5_000)).await.unwrap();

    let rows = list_for_task(db, task.id).await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].used_tokens, 5_000);
}

#[tokio::test]
async fn the_row_of_a_task_with_no_step_and_the_row_of_a_step_coexist() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let (task, step) = a_task_with_a_step(db).await;

    upsert(db, fill(task.id, None, 2_000)).await.unwrap();
    upsert(db, fill(task.id, Some(step.id), 3_000)).await.unwrap();
    upsert(db, fill(task.id, None, 4_000)).await.unwrap();

    assert_eq!(find(db, task.id, None).await.unwrap().unwrap().used_tokens, 4_000);
    assert_eq!(find(db, task.id, Some(step.id)).await.unwrap().unwrap().used_tokens, 3_000);
    assert_eq!(list_for_task(db, task.id).await.unwrap().len(), 2);
}

#[tokio::test]
async fn deleting_a_step_removes_its_row_and_keeps_the_tasks() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let (task, step) = a_task_with_a_step(db).await;
    upsert(db, fill(task.id, None, 2_000)).await.unwrap();
    upsert(db, fill(task.id, Some(step.id), 3_000)).await.unwrap();

    task_step::Entity::delete_by_id(step.id).exec(db).await.unwrap();

    let rows = list_for_task(db, task.id).await.unwrap();
    assert_eq!(rows.iter().map(|row| row.step_id).collect::<Vec<_>>(), [None]);
}

#[tokio::test]
async fn deleting_a_task_removes_both_rows() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let (task, step) = a_task_with_a_step(db).await;
    upsert(db, fill(task.id, None, 2_000)).await.unwrap();
    upsert(db, fill(task.id, Some(step.id), 3_000)).await.unwrap();

    task::Entity::delete_by_id(task.id).exec(db).await.unwrap();

    assert!(Entity::find().all(db).await.unwrap().is_empty());
}
