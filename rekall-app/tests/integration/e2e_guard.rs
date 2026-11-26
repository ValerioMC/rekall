//! The context a guard hook asks Rekall for: the project that holds a folder, whether it commits
//! for the session, the step that is running and the credential locations a session must not touch.

mod support;

use std::path::Path;
use std::process::{Output, Stdio};

use rekall_core::claude::SessionProvider;
use serde_json::json;
use support::{a_git_repo_with_one_commit, app, App};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

fn context_url(folder: &Path) -> String {
    format!("/api/guard/context?folder={}", folder.display())
}

async fn stop_hook(app: &App, folder: &Path, scope: &str, provider: SessionProvider) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_rekall-server"));
    command.arg("guard");
    if provider == SessionProvider::Codex {
        command.arg("codex");
    }
    let mut hook = command.arg("before-stop")
        .env("REKALL_HOME", app.folder.path().join("home/.rekall"))
        .env("REKALL_GUARD_PORT", app.running.as_ref().unwrap().port.to_string())
        .env("REKALL_GUARD_SCOPE", scope)
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped())
        .spawn().unwrap();
    let event = json!({ "cwd": folder, "hook_event_name": "Stop", "stop_hook_active": false });
    hook.stdin.take().unwrap().write_all(event.to_string().as_bytes()).await.unwrap();
    hook.wait_with_output().await.unwrap()
}

#[tokio::test]
async fn guards_scope_work_to_one_task_and_skip_preparation_and_unscoped_project_stops() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let repo = a_git_repo_with_one_commit("Start");
    let project_id = app.a_project_in(&acme, "vega", repo.path()).await;
    app.put(
        &format!("/api/projects/{project_id}"),
        json!({ "label": "vega", "title": "Vega", "status": "ACTIVE", "companyId": acme, "repoFolder": repo.path().to_string_lossy(), "autoCommit": true }),
    )
    .await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    app.a_step(&task_id, "First", None).await;
    app.a_step(&task_id, "Second", None).await;
    app.call_tool("rekall_step", json!({ "anchors": "task:VE-1", "step": "2", "state": "running" })).await;

    let other_id = app.a_task(&project_id, "parallel").await;
    app.a_step(&other_id, "Independent", None).await;
    app.call_tool("rekall_step", json!({ "anchors": "task:VE-2", "step": "1", "state": "running" })).await;
    let worktree = app.folder.path().join("vega-report-worktree");
    std::fs::create_dir(&worktree).unwrap();
    app.execute("UPDATE task SET worktree_folder = ? WHERE id = ?", vec![worktree.to_string_lossy().into_owned().into(), task_id.clone().into()]).await;

    let folder = context_url(&repo.path().join("src"));
    let context = app.get(&format!("{folder}&scope={task_id}")).await;

    assert_eq!(context.status, 200);
    assert_eq!(context.str("projectAnchor"), "project:vega");
    assert_eq!(context.get("autoCommit"), &json!(true));
    assert_eq!(context.get("runningStep"), &json!({ "number": 2, "taskAnchor": "task:VE-1" }));
    assert_eq!(app.get(&format!("{folder}&scope={other_id}")).await.get("runningStep"), &json!({ "number": 1, "taskAnchor": "task:VE-2" }));
    assert_eq!(app.get(&folder).await.get("runningStep"), &json!(null));

    let worktree_context = context_url(&worktree.join("src"));
    assert_eq!(app.get(&worktree_context).await.get("runningStep"), context.get("runningStep"));
    let preparation = app.get(&format!("{worktree_context}&scope=without-step")).await;
    assert_eq!(preparation.get("runningStep"), &json!(null));
    assert_eq!(preparation.get("autoCommit"), &json!(true));

    for provider in [SessionProvider::Claude, SessionProvider::Codex] {
        let refusal = stop_hook(&app, &worktree, &task_id, provider).await;
        let reason = if provider == SessionProvider::Codex {
            assert!(refusal.status.success());
            assert!(refusal.stderr.is_empty());
            let response: serde_json::Value = serde_json::from_slice(&refusal.stdout).unwrap();
            assert_eq!(response["decision"], "block");
            response["reason"].as_str().unwrap().to_owned()
        } else {
            assert_eq!(refusal.status.code(), Some(2));
            String::from_utf8(refusal.stderr).unwrap()
        };
        assert!(reason.contains("/rk task:VE-1 step:2 done"), "{reason}");
        let allowed = stop_hook(&app, &worktree, "without-step", provider).await;
        assert!(allowed.status.success());
        assert!(allowed.stdout.is_empty());
        assert!(allowed.stderr.is_empty());
    }

    app.execute("UPDATE task_step SET state = 'CLAIMED' WHERE task_id = ? AND state = 'RUNNING'", vec![task_id.clone().into()]).await;
    assert_eq!(app.get(&format!("{folder}&scope={task_id}")).await.get("runningStep"), &json!(null));
    assert_eq!(app.get(&worktree_context).await.get("runningStep"), &json!(null));
}

#[tokio::test]
async fn a_folder_no_project_holds_has_no_project_and_still_lists_the_protected_credentials() {
    let app = app().await;

    let context = app.get("/api/guard/context?folder=/nowhere/at/all").await;

    assert_eq!(context.status, 200);
    assert_eq!(context.get("projectAnchor"), &json!(null));
    assert_eq!(context.get("autoCommit"), &json!(false));
    assert_eq!(context.get("runningStep"), &json!(null));
    let references = context.get("protectedCredentialReferences").as_array().unwrap().clone();
    assert!(references.iter().any(|reference| reference.as_str().unwrap().ends_with("secrets.json")), "{references:?}");
}

#[tokio::test]
async fn a_request_without_a_folder_is_refused() {
    let app = app().await;

    assert_eq!(app.get("/api/guard/context").await.status, 400);
}

#[tokio::test]
async fn the_protections_are_installed_into_claude_codes_settings_and_removed_again() {
    let app = app().await;
    let settings = app.folder.path().join("home/.claude/settings.json");
    std::fs::create_dir_all(settings.parent().unwrap()).unwrap();
    std::fs::write(&settings, r#"{"model":"opus"}"#).unwrap();
    assert_eq!(app.get("/api/guard/status").await.str("state"), "NOT_INSTALLED");

    let installed = app.post_empty("/api/guard/install").await;

    assert_eq!((installed.status, installed.str("state")), (200, "INSTALLED".into()));
    let written: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&settings).unwrap()).unwrap();
    assert_eq!(written["model"], "opus");
    assert!(written["hooks"]["Stop"][0]["hooks"][0]["command"].as_str().unwrap().ends_with(" guard before-stop"));

    let removed = app.post_empty("/api/guard/uninstall").await;

    assert_eq!((removed.status, removed.str("state")), (200, "NOT_INSTALLED".into()));
    assert_eq!(std::fs::read_to_string(&settings).unwrap().replace(char::is_whitespace, ""), r#"{"model":"opus"}"#);
}

#[tokio::test]
async fn a_settings_file_that_is_not_json_is_refused_with_a_400_and_left_alone() {
    let app = app().await;
    let settings = app.folder.path().join("home/.claude/settings.json");
    std::fs::create_dir_all(settings.parent().unwrap()).unwrap();
    std::fs::write(&settings, "{ nope").unwrap();

    assert_eq!(app.post_empty("/api/guard/install").await.status, 400);
    assert_eq!(app.get("/api/guard/status").await.str("state"), "INVALID_SETTINGS");
    assert_eq!(std::fs::read_to_string(&settings).unwrap(), "{ nope");
}
