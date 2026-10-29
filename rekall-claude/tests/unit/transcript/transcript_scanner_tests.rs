use std::fs;

use rekall_kernel::{Id, Instant};
use rekall_store::{company, project, task, ProjectStatus};
use rekall_core::{Ctx, EventBus, Services};
use sea_orm::{ActiveModelTrait, IntoActiveModel};

use super::*;

const SESSION: &str = concat!(
    r#"{"type":"user","timestamp":"2026-10-01T10:00:00Z","message":{"content":"/rk project:vega task:report"}}"#,
    "\n",
    r#"{"type":"assistant","timestamp":"2026-10-01T10:00:01Z","message":{"id":"m1","content":[],"usage":{"input_tokens":1,"output_tokens":40,"cache_creation_input_tokens":2,"cache_read_input_tokens":3}}}"#,
    "\n"
);

async fn services_with_project(folder: &Path) -> (rekall_store::Database, Services) {
    let database = rekall_store::open_temporary().await.unwrap();
    let services = Services::new(Ctx::new(database.conn.clone(), EventBus::new()));
    let now = Instant::now();
    let company = company::Model { id: Id::random(), name: "Acme".into(), description: None, created_at: now, updated_at: now };
    company.clone().into_active_model().insert(&database.conn).await.unwrap();
    let project = project::Model {
        id: Id::random(), label: "vega".into(), title: "Vega".into(), status: ProjectStatus::Active, icon: "folder".into(),
        description: None, blueprint_markdown: None, repo_folder: Some(folder.to_string_lossy().into_owned()),
        auto_commit: false, company_id: company.id, created_at: now, updated_at: now,
    };
    project.clone().into_active_model().insert(&database.conn).await.unwrap();
    task::Model::new("report".into(), "Report".into(), project.id, now).into_active_model().insert(&database.conn).await.unwrap();
    (database, services)
}

#[tokio::test]
async fn a_session_is_read_once_until_its_transcript_grows() {
    let root = tempfile::tempdir().unwrap();
    let transcript = root.path().join("-work-vega/s1.jsonl");
    fs::create_dir_all(transcript.parent().unwrap()).unwrap();
    fs::write(&transcript, SESSION).unwrap();
    let (_database, services) = services_with_project(Path::new("/work/vega")).await;
    let scanner = TranscriptScanner::new(TranscriptFolder::new(root.path().to_path_buf()), services.token_usage.clone());

    assert_eq!(scanner.scan().await.unwrap(), 1);
    assert_eq!(scanner.scan().await.unwrap(), 0);
    let grown = SESSION.to_string() + &SESSION.lines().nth(1).unwrap().replace("m1", "m2") + "\n";
    fs::write(&transcript, grown).unwrap();
    assert_eq!(scanner.scan().await.unwrap(), 1);

    let tallies = services.token_usage.tallies().await.unwrap();
    assert_eq!(tallies.len(), 1);
    assert_eq!((tallies[0].output_tokens, tallies[0].cache_read_tokens), (80, 6));
}

#[tokio::test]
async fn nothing_is_read_while_no_project_has_a_folder_with_sessions() {
    let root = tempfile::tempdir().unwrap();
    let (_database, services) = services_with_project(Path::new("/work/vega")).await;
    let scanner = TranscriptScanner::new(TranscriptFolder::new(root.path().join("missing")), services.token_usage.clone());

    assert_eq!(scanner.scan().await.unwrap(), 0);
    assert!(services.token_usage.tallies().await.unwrap().is_empty());
}
