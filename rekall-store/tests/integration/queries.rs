//! The queries whose rules live in SQL: label lookups, catalog order, and note search.

use rekall_kernel::{Id, Instant};
use rekall_store::{company, document, project, task, DocumentContextMode, ProjectStatus};
use rekall_store::repository as repo;
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
        description: None, blueprint_markdown: None, repo_folder: None, auto_commit: false, main_branch: None, pull_request_flow: false, task_key: String::new(), company_id: company.id,
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
        id: Id::random(), task_id: on.id, position: 0, title: title.into(), kind: "notes".into(), body_markdown: body.into(),
        source_path: None, context_mode: DocumentContextMode::Full, created_at: now, updated_at: now,
    };
    note.clone().into_active_model().insert(db).await.unwrap();
    note
}

#[tokio::test]
async fn a_code_finds_its_task_whatever_the_case_it_is_typed_in() {
    let database = rekall_store::open_temporary().await.unwrap();
    let db = &database.conn;
    let acme = a_company(db, "Acme").await;
    let vega = a_project(db, &acme, "vega").await;
    let mut report = task::Model::new("report-builder".into(), "Report".into(), vega.id, Instant::now());
    report.assign_code("VG", 1);
    report.clone().into_active_model().insert(db).await.unwrap();

    assert_eq!(repo::task::find_by_code(db, "vg-1").await.unwrap(), Some(report));
    assert_eq!(repo::task::find_by_code(db, "VG-2").await.unwrap(), None);
    assert_eq!(repo::project::find_by_anchor(db, "acme", "Vega").await.unwrap(), Some(vega));
}

#[tokio::test]
async fn the_next_number_is_one_past_the_highest_of_the_project() {
    let database = rekall_store::open_temporary().await.unwrap();
    let db = &database.conn;
    let acme = a_company(db, "Acme").await;
    let vega = a_project(db, &acme, "vega").await;
    assert_eq!(repo::task::next_number(db, vega.id).await.unwrap(), 1);
    for number in [1, 2, 5] {
        let mut task = task::Model::new(format!("task-{number}"), "Task".into(), vega.id, Instant::now());
        task.assign_code("VG", number);
        task.into_active_model().insert(db).await.unwrap();
    }
    assert_eq!(repo::task::next_number(db, vega.id).await.unwrap(), 6);
}

#[tokio::test]
async fn changing_a_key_writes_every_code_of_the_project_again() {
    let database = rekall_store::open_temporary().await.unwrap();
    let db = &database.conn;
    let acme = a_company(db, "Acme").await;
    let vega = a_project(db, &acme, "vega").await;
    let mut task = task::Model::new("report".into(), "Report".into(), vega.id, Instant::now());
    task.assign_code("VG", 3);
    task.clone().into_active_model().insert(db).await.unwrap();

    repo::task::recode_project(db, vega.id, "VEGA").await.unwrap();

    assert_eq!(repo::task::require(db, task.id).await.unwrap().code, "VEGA-3");
}

#[tokio::test]
async fn the_catalog_lists_projects_by_company_name_then_label() {
    let database = rekall_store::open_temporary().await.unwrap();
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
    let database = rekall_store::open_temporary().await.unwrap();
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
async fn a_projects_notes_are_those_on_its_tasks() {
    let database = rekall_store::open_temporary().await.unwrap();
    let db = &database.conn;
    let acme = a_company(db, "Acme").await;
    let vega = a_project(db, &acme, "vega").await;
    let other = a_project(db, &acme, "other").await;
    let first = a_task(db, &vega, "first").await;
    let second = a_task(db, &vega, "second").await;
    let elsewhere = a_task(db, &other, "elsewhere").await;
    a_note(db, "First", "x", &first).await;
    a_note(db, "Second", "x", &second).await;
    a_note(db, "Elsewhere", "x", &elsewhere).await;

    let mut notes: Vec<String> = repo::document::list_on_project_tasks(db, vega.id).await.unwrap().into_iter().map(|d| d.title).collect();
    notes.sort();

    assert_eq!(notes, ["First", "Second"]);
}
