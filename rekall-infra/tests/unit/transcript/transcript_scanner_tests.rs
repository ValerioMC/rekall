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
    let services = Services::new(Ctx::new(database.conn.clone(), EventBus::new()), std::sync::Arc::new(rekall_core::check::ScriptedCheckRunner::new()));
    let now = Instant::now();
    let company = company::Model { id: Id::random(), name: "Acme".into(), description: None, created_at: now, updated_at: now };
    company.clone().into_active_model().insert(&database.conn).await.unwrap();
    let project = project::Model {
        id: Id::random(), label: "vega".into(), title: "Vega".into(), status: ProjectStatus::Active, icon: "folder".into(),
        description: None, blueprint_markdown: None, repo_folder: Some(folder.to_string_lossy().into_owned()),
        auto_commit: false, main_branch: None, pull_request_flow: false, task_key: String::new(), company_id: company.id, created_at: now, updated_at: now,
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

#[derive(Clone)]
struct CodexHome(PathBuf);

#[async_trait::async_trait]
impl crate::cli::EnvironmentSource for CodexHome {
    async fn current(&self) -> std::collections::HashMap<String, String> {
        [("CODEX_HOME".into(), self.0.to_string_lossy().into_owned())].into()
    }
}

#[tokio::test]
async fn codex_rollouts_replace_the_same_thread_after_a_restart_and_skip_other_folders() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("codex");
    let active = home.join("sessions/2026/11/26");
    let archive = home.join("archived_sessions");
    fs::create_dir_all(&active).unwrap();
    fs::create_dir_all(&archive).unwrap();
    let saved = active.join("rollout-1.jsonl");
    let older = archive.join("rollout-1.jsonl");
    let header = r#"{"type":"session_meta","payload":{"id":"session-1","cwd":"/work/vega"}}"#;
    let prompt = r#"{"type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"/rk project:vega task:report"}]}}"#;
    let count = r#"{"timestamp":"2026-11-26T15:10:35Z","type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"input_tokens":100,"cached_input_tokens":20,"output_tokens":30,"total_tokens":130}}}}"#;
    let original = format!("{header}\n{prompt}\n{count}\n");
    fs::write(&saved, &original).unwrap();
    fs::write(&older, &original).unwrap();
    fs::write(active.join("rollout-other.jsonl"), original.replace("session-1", "other").replace("/work/vega", "/work/vega-old")).unwrap();
    let (_database, services) = services_with_project(Path::new("/work/vega")).await;
    let scanner = || TranscriptScanner::with_codex(
        TranscriptFolder::new(root.path().join("claude")),
        services.token_usage.clone(),
        CodexCli::new(root.path().to_path_buf(), Arc::new(CodexHome(home.clone()))),
        root.path().join("worktrees"),
    );

    assert_eq!(scanner().scan().await.unwrap(), 1);
    assert_eq!(scanner().scan().await.unwrap(), 0);
    let grown = count.replace("\"output_tokens\":30", "\"output_tokens\":50").replace("\"total_tokens\":130", "\"total_tokens\":150");
    fs::write(&saved, format!("{original}{grown}\n")).unwrap();
    assert_eq!(scanner().scan().await.unwrap(), 1);

    let tallies = services.token_usage.tallies().await.unwrap();
    assert_eq!(tallies.len(), 1);
    assert_eq!(tallies[0].provider, rekall_store::SessionProvider::Codex);
    assert_eq!((tallies[0].input_tokens, tallies[0].output_tokens, tallies[0].cache_read_tokens), (80, 50, 20));
}
