use super::*;
use crate::enums::{DocumentContextMode, ProjectStatus, WrapupAuthor};
use crate::{company, document, project, task, task_step, tracker_picture};
use rekall_kernel::{Id, Instant};
use sea_orm::{ActiveModelTrait, IntoActiveModel};

const FRAGMENT: &str = "/api/images/";

async fn a_task(db: &impl ConnectionTrait) -> task::Model {
    let now = Instant::now();
    let company = company::Model { id: Id::random(), name: "Acme".into(), description: None, created_at: now, updated_at: now };
    company.clone().into_active_model().insert(db).await.unwrap();
    let project = project::Model {
        id: Id::random(), label: "web".into(), title: "Web".into(), status: ProjectStatus::Active, icon: "folder".into(),
        description: None, blueprint_markdown: None, repo_folder: None, auto_commit: false, main_branch: None, pull_request_flow: false, task_key: String::new(), company_id: company.id,
        created_at: now, updated_at: now,
    };
    project.clone().into_active_model().insert(db).await.unwrap();
    let task = task::Model::new("web-12".into(), "Login".into(), project.id, now);
    task.clone().into_active_model().insert(db).await.unwrap();
    task
}

fn a_note(task_id: Id, position: i32, body: &str) -> document::Model {
    let now = Instant::now();
    document::Model {
        id: Id::random(), task_id, position, title: format!("note-{position}"), kind: "notes".into(), body_markdown: body.into(),
        source_path: None, context_mode: DocumentContextMode::Full, created_at: now, updated_at: now,
    }
}

async fn table(db: &impl ConnectionTrait, name: &str) -> TextTable {
    tables(db).await.unwrap().into_iter().find(|table| table.name == name).unwrap()
}

async fn every_text(db: &impl ConnectionTrait) -> Vec<String> {
    let mut found = Vec::new();
    for table in tables(db).await.unwrap() {
        found.extend(page_holding(db, &table, FRAGMENT, None, 1_000).await.unwrap().texts);
    }
    found
}

#[tokio::test]
async fn every_table_of_the_schema_is_listed_and_readable_without_the_bookkeeping_ones() {
    let database = crate::open_temporary().await.unwrap();

    let listed = tables(&database.conn).await.unwrap();

    let names: Vec<&str> = listed.iter().map(|table| table.name.as_str()).collect();
    assert!(["task", "task_step", "document", "wrapup_version", "project_section", "tracker_picture"].iter().all(|name| names.contains(name)), "{names:?}");
    assert!(!names.iter().any(|name| name.starts_with("sqlite_") || *name == "seaql_migrations"), "{names:?}");
    for table in &listed {
        row_count(&database.conn, table).await.unwrap();
        page_holding(&database.conn, table, FRAGMENT, None, 1).await.unwrap();
    }
}

#[tokio::test]
async fn paging_reads_every_row_once() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let task = a_task(db).await;
    for position in 0..5 {
        a_note(task.id, position, &format!("![shot](/api/images/acme/web/web-12/{position}.png)")).into_active_model().insert(db).await.unwrap();
    }
    let notes = table(db, "document").await;

    let mut read = Vec::new();
    let mut after_row = None;
    loop {
        let page = page_holding(db, &notes, FRAGMENT, after_row, 2).await.unwrap();
        if page.rows == 0 {
            break;
        }
        after_row = page.last_row;
        read.extend(page.texts);
    }

    read.sort();
    assert_eq!(read, (0..5).map(|position| format!("![shot](/api/images/acme/web/web-12/{position}.png)")).collect::<Vec<_>>());
    assert_eq!(row_count(db, &notes).await.unwrap(), 5);
}

#[tokio::test]
async fn a_link_is_found_in_a_note_a_wrapup_version_and_the_earlier_pass_of_a_step() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let task = a_task(db).await;
    a_note(task.id, 0, "see /api/images/acme/web/web-12/note.png.").into_active_model().insert(db).await.unwrap();
    let version = crate::wrapup::wrapup_version::Model {
        id: Id::random(), task_id: task.id, number: 1, body_markdown: "![old](/api/images/acme/web/web-12/old.png)".into(),
        written_by: WrapupAuthor::Claude, written_at: Instant::now(), commit_hash: None,
    };
    version.into_active_model().insert(db).await.unwrap();
    let mut step = task_step::Model::new(task.id, "Build".into(), 0, Instant::now());
    step.passes_json = r#"[{"detailMarkdown":"![first](/api/images/acme/web/web-12/pass.png)","sentBackAt":"2026-01-01T00:00:00Z"}]"#.into();
    step.into_active_model().insert(db).await.unwrap();

    let found = every_text(db).await.join("\n");

    for name in ["note.png", "old.png", "pass.png"] {
        assert!(found.contains(&format!("/api/images/acme/web/web-12/{name}")), "{name} in {found}");
    }
}

#[tokio::test]
async fn texts_without_the_fragment_and_the_upload_memory_are_left_out() {
    let database = crate::open_temporary().await.unwrap();
    let db = &database.conn;
    let task = a_task(db).await;
    a_note(task.id, 0, "no picture here").into_active_model().insert(db).await.unwrap();
    let uploaded = tracker_picture::Model {
        task_id: task.id, destination: "https://github.com/acme/web".into(), picture_path: "acme/web/web-12/login.png".into(),
        remote_reference: "https://github.com/user-attachments/assets/1".into(), created_at: Instant::now(),
    };
    uploaded.into_active_model().insert(db).await.unwrap();

    assert_eq!(every_text(db).await, Vec::<String>::new());
}
