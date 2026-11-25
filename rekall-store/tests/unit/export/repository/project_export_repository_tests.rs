use super::*;
use crate::enums::{ExportFailureKind, ExportFormat, ProjectStatus};
use rekall_kernel::Instant;
use sea_orm::{ActiveModelTrait, IntoActiveModel};

async fn a_project(db: &impl ConnectionTrait, company_name: &str, label: &str) -> project::Model {
    let now = Instant::now();
    let company = company::Model { id: Id::random(), name: company_name.into(), description: None, created_at: now, updated_at: now };
    company.clone().into_active_model().insert(db).await.unwrap();
    let project = project::Model {
        id: Id::random(), label: label.into(), title: label.to_uppercase(), status: ProjectStatus::Active, icon: "folder".into(),
        description: None, blueprint_markdown: None, repo_folder: None, auto_commit: false, main_branch: None, pull_request_flow: false,
        task_key: label.to_uppercase(), company_id: company.id, created_at: now, updated_at: now,
    };
    project.clone().into_active_model().insert(db).await.unwrap();
    project
}

fn done_export(project_id: Id, created_at: Instant) -> Model {
    Model {
        id: Id::random(), project_id, format: ExportFormat::Pdf, status: ExportStatus::Done, summary: "Blueprint, 3 pages".into(),
        file_name: "vega-blueprint.pdf".into(), size_bytes: 4, diagrams_missing: 1, failure_kind: None, failure_message: None, created_at,
    }
}

fn failed_export(project_id: Id, created_at: Instant) -> Model {
    Model {
        status: ExportStatus::Failed, size_bytes: 0, failure_kind: Some(ExportFailureKind::Render),
        failure_message: Some("The PDF could not be written: no glyph".into()), ..done_export(project_id, created_at)
    }
}

#[tokio::test]
async fn a_done_export_is_stored_with_its_file() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let project = a_project(db, "Acme", "vega").await;
    let export = done_export(project.id, Instant::now());

    insert_done(db, export.clone(), vec![1, 2, 3, 4]).await.unwrap();

    assert_eq!(require(db, export.id).await.unwrap(), export);
    assert_eq!(file_of(db, export.id).await.unwrap(), Some(vec![1, 2, 3, 4]));
}

#[tokio::test]
async fn a_failed_export_is_stored_with_its_kind_and_has_no_file() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let project = a_project(db, "Acme", "vega").await;
    let export = failed_export(project.id, Instant::now());

    insert_failed(db, export.clone()).await.unwrap();

    let stored = require(db, export.id).await.unwrap();
    assert_eq!((stored.status, stored.failure_kind), (ExportStatus::Failed, Some(ExportFailureKind::Render)));
    assert_eq!(file_of(db, export.id).await.unwrap(), None);
}

#[tokio::test]
async fn a_failed_export_without_its_message_is_refused() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let project = a_project(db, "Acme", "vega").await;
    let export = Model { failure_message: None, ..failed_export(project.id, Instant::now()) };

    let refusal = insert_failed(db, export).await.unwrap_err();

    assert!(refusal.is_illegal_argument());
    assert!(list_with_project_and_company(db).await.unwrap().is_empty());
}

#[tokio::test]
async fn an_export_is_stored_only_by_the_write_of_its_status() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let project = a_project(db, "Acme", "vega").await;

    assert!(insert_done(db, failed_export(project.id, Instant::now()), vec![1]).await.unwrap_err().is_illegal_argument());
    assert!(insert_failed(db, done_export(project.id, Instant::now())).await.unwrap_err().is_illegal_argument());
}

#[tokio::test]
async fn the_list_is_newest_first_with_the_project_and_company_names() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let vega = a_project(db, "Acme", "vega").await;
    let orion = a_project(db, "Globex", "orion").await;
    let older = done_export(vega.id, Instant::now());
    let newer = failed_export(orion.id, Instant::now());
    insert_done(db, older.clone(), vec![1]).await.unwrap();
    insert_failed(db, newer.clone()).await.unwrap();

    let listing = list_with_project_and_company(db).await.unwrap();

    let rows: Vec<(Id, &str, &str)> = listing.iter().map(|row| (row.export.id, row.project_title.as_str(), row.company_name.as_str())).collect();
    assert_eq!(rows, [(newer.id, "ORION", "Globex"), (older.id, "VEGA", "Acme")]);
    assert_eq!(listing[0].company_id, orion.company_id);
}

#[tokio::test]
async fn deleting_an_export_removes_its_file() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let project = a_project(db, "Acme", "vega").await;
    let export = done_export(project.id, Instant::now());
    insert_done(db, export.clone(), vec![1]).await.unwrap();

    delete(db, export.id).await.unwrap();

    assert!(find_by_id(db, export.id).await.unwrap().is_none());
    assert!(project_export_file::Entity::find().all(db).await.unwrap().is_empty());
}

#[tokio::test]
async fn deleting_the_project_removes_its_exports_and_their_files() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let project = a_project(db, "Acme", "vega").await;
    insert_done(db, done_export(project.id, Instant::now()), vec![1]).await.unwrap();
    insert_failed(db, failed_export(project.id, Instant::now())).await.unwrap();

    project::Entity::delete_by_id(project.id).exec(db).await.unwrap();

    assert!(Entity::find().all(db).await.unwrap().is_empty());
    assert!(project_export_file::Entity::find().all(db).await.unwrap().is_empty());
}

#[tokio::test]
async fn an_unknown_export_is_not_found() {
    let database = crate::open_temporary().await.unwrap();

    let refusal = require(&database.conn, Id::random()).await.unwrap_err();

    assert!(refusal.message().contains("Export"));
}

#[tokio::test]
async fn deleting_matching_exports_keeps_the_others() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let vega = a_project(db, "Acme", "vega").await;
    let orion = a_project(db, "Globex", "orion").await;
    let kept = done_export(vega.id, Instant::now());
    insert_done(db, kept.clone(), vec![1]).await.unwrap();
    insert_failed(db, failed_export(vega.id, Instant::now())).await.unwrap();
    insert_done(db, done_export(orion.id, Instant::now()), vec![2]).await.unwrap();
    insert_failed(db, failed_export(orion.id, Instant::now())).await.unwrap();

    assert_eq!(delete_matching(db, None, None, Some(ExportStatus::Failed)).await.unwrap(), 2);
    assert_eq!(delete_matching(db, None, Some(vega.id), None).await.unwrap(), 1);
    assert_eq!(delete_matching(db, Some(orion.company_id), None, None).await.unwrap(), 1);

    assert!(Entity::find().all(db).await.unwrap().is_empty());
    assert!(project_export_file::Entity::find().all(db).await.unwrap().is_empty());
}

#[tokio::test]
async fn deleting_by_company_and_status_takes_only_what_matches_both() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let vega = a_project(db, "Acme", "vega").await;
    let orion = a_project(db, "Globex", "orion").await;
    insert_done(db, done_export(vega.id, Instant::now()), vec![1]).await.unwrap();
    insert_failed(db, failed_export(vega.id, Instant::now())).await.unwrap();
    insert_failed(db, failed_export(orion.id, Instant::now())).await.unwrap();

    let deleted = delete_matching(db, Some(vega.company_id), None, Some(ExportStatus::Failed)).await.unwrap();

    assert_eq!(deleted, 1);
    assert_eq!(Entity::find().all(db).await.unwrap().len(), 2);
}
