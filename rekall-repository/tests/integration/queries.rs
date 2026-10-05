//! The queries whose rules live in SQL: label lookups, catalog order, and note search.

use rekall_common::{Id, Instant};
use rekall_model::{company, document, document_task, project, task, DocumentContextMode, ProjectStatus};
use rekall_repository::repository as repo;
use sea_orm::{ActiveModelTrait, ConnectionTrait, IntoActiveModel};

async fn a_company(db: &impl ConnectionTrait, name: &str) -> company::Model {
    let now = Instant::now();
    let company = company::Model { id: Id::random(), name: name.into(), description: None, created_at: now, updated_at: now };
    company.clone().into_active_model().insert(db).await.unwrap();
    company
}

async fn a_project(db: &impl ConnectionTrait, company: &company::Model, label: &str) -> project::Model {
    let now = Instant::now();
    let project = project::Model {
        id: Id::random(), label: label.into(), title: label.into(), status: ProjectStatus::Active, icon: "folder".into(),
        description: None, blueprint_markdown: None, repo_folder: None, auto_commit: false, company_id: company.id,
        created_at: now, updated_at: now,
    };
    project.clone().into_active_model().insert(db).await.unwrap();
    project
}

async fn a_task(db: &impl ConnectionTrait, project: &project::Model, label: &str) -> task::Model {
    let task = task::Model::new(label.into(), label.into(), project.id, Instant::now());
    task.clone().into_active_model().insert(db).await.unwrap();
    task
}

async fn a_note(db: &impl ConnectionTrait, title: &str, body: &str, on: &task::Model) -> document::Model {
    let now = Instant::now();
    let note = document::Model {
        id: Id::random(), title: title.into(), kind: "notes".into(), body_markdown: body.into(), source_path: None,
        context_mode: DocumentContextMode::Full, scope_company_id: None, scope_project_id: None, created_at: now, updated_at: now,
    };
    note.clone().into_active_model().insert(db).await.unwrap();
    document_task::Model { document_id: note.id, task_id: on.id, position: 0 }.into_active_model().insert(db).await.unwrap();
    note
}

#[tokio::test]
async fn an_anchor_finds_its_task_whatever_the_case_it_is_typed_in() {
    let database = rekall_repository::open_temporary().await.unwrap();
    let db = &database.conn;
    let acme = a_company(db, "Acme").await;
    let vega = a_project(db, &acme, "vega").await;
    let report = a_task(db, &vega, "report-builder").await;

    assert_eq!(repo::task::find_by_anchor(db, "VEGA", "Report-Builder").await.unwrap(), Some(report.clone()));
    assert_eq!(repo::task::find_by_label(db, "REPORT-builder").await.unwrap(), vec![report]);
    assert_eq!(repo::project::find_by_anchor(db, "acme", "Vega").await.unwrap(), Some(vega));
    assert_eq!(repo::task::find_by_anchor(db, "vega", "nothing").await.unwrap(), None);
}

#[tokio::test]
async fn the_catalog_lists_projects_by_company_name_then_label() {
    let database = rekall_repository::open_temporary().await.unwrap();
    let db = &database.conn;
    let globex = a_company(db, "Globex").await;
    let acme = a_company(db, "Acme").await;
    a_project(db, &globex, "alpha").await;
    a_project(db, &acme, "zeta").await;
    a_project(db, &acme, "beta").await;

    let order: Vec<String> = repo::project::list_in_catalog_order(db).await.unwrap().into_iter().map(|p| p.label).collect();

    assert_eq!(order, ["beta", "zeta", "alpha"]);
}

#[tokio::test]
async fn note_search_takes_percent_and_underscore_literally() {
    let database = rekall_repository::open_temporary().await.unwrap();
    let db = &database.conn;
    let acme = a_company(db, "Acme").await;
    let vega = a_project(db, &acme, "vega").await;
    let task = a_task(db, &vega, "report").await;
    a_note(db, "Budget", "Spend 100% of it", &task).await;
    a_note(db, "Cluster", "Nothing about money", &task).await;

    let found: Vec<String> = repo::document::search_by_title(db, "100%").await.unwrap().into_iter().map(|d| d.title).collect();
    let wildcard: Vec<String> = repo::document::search_by_title(db, "c_uster").await.unwrap().into_iter().map(|d| d.title).collect();

    assert_eq!(found, ["Budget"]);
    assert!(wildcard.is_empty(), "an underscore matches only an underscore");
}

#[tokio::test]
async fn a_projects_notes_are_those_on_its_tasks_once_each() {
    let database = rekall_repository::open_temporary().await.unwrap();
    let db = &database.conn;
    let acme = a_company(db, "Acme").await;
    let vega = a_project(db, &acme, "vega").await;
    let other = a_project(db, &acme, "other").await;
    let first = a_task(db, &vega, "first").await;
    let second = a_task(db, &vega, "second").await;
    let elsewhere = a_task(db, &other, "elsewhere").await;
    let shared = a_note(db, "Shared", "x", &first).await;
    document_task::Model { document_id: shared.id, task_id: second.id, position: 0 }.into_active_model().insert(db).await.unwrap();
    a_note(db, "Elsewhere", "x", &elsewhere).await;

    let notes: Vec<String> = repo::document::list_on_project_tasks(db, vega.id).await.unwrap().into_iter().map(|d| d.title).collect();

    assert_eq!(notes, ["Shared"]);
}
