//! The context a guard hook asks Rekall for: the project that holds a folder, whether it commits
//! for the session, the step that is running and the credential locations a session must not touch.

mod support;

use serde_json::json;
use support::{a_git_repo_with_one_commit, app};

fn context_url(folder: &std::path::Path) -> String {
    format!("/api/guard/context?folder={}", folder.display())
}

#[tokio::test]
async fn a_folder_inside_an_auto_commit_project_answers_its_commit_rule_and_its_running_step() {
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
    app.call_tool("rekall_step", json!({ "anchors": "project:vega task:report-builder", "step": "2", "state": "running" })).await;

    let context = app.get(&context_url(&repo.path().join("src"))).await;

    assert_eq!(context.status, 200);
    assert_eq!(context.str("projectAnchor"), "project:vega");
    assert_eq!(context.get("autoCommit"), &json!(true));
    assert_eq!(context.get("runningStep"), &json!({ "number": 2, "taskAnchor": "project:vega task:report-builder" }));
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
