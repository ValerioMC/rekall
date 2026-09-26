//! `RekallEndToEndTest`, part two: commit references and auto-commit, against real git
//! repositories.

mod support;

use serde_json::{json, Value};
use support::{a_git_repo_with_commits, a_git_repo_with_one_commit, app, run_git, uuid_value, App};

async fn update_project(app: &App, project_id: &str, company_id: &str, folder: &str, auto_commit: bool) -> Value {
    app.put(
        &format!("/api/projects/{project_id}"),
        json!({ "label": "vega", "title": "Vega", "status": "ACTIVE", "companyId": company_id, "repoFolder": folder, "autoCommit": auto_commit }),
    )
    .await
    .body
}

#[tokio::test]
async fn rekall_record_commit_reads_the_projects_own_repo_folder_and_logs_its_tip_commit() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let repo = a_git_repo_with_one_commit("Wire up the commit ledger");
    let project_id = app.a_project_in(&acme, "vega", repo.path()).await;
    let task_id = app.a_task(&project_id, "report-builder").await;

    let logged = app.call_tool("rekall_record_commit", json!({ "anchors": "project:vega task:report-builder" })).await;

    assert_contains!(logged, "Wire up the commit ledger", "the task");
    assert_eq!(
        app.string("SELECT comment FROM commit_reference WHERE task_id = ? AND step_id IS NULL", vec![uuid_value(&task_id)]).await,
        Some("Wire up the commit ledger".into())
    );
}

#[tokio::test]
async fn the_console_button_logs_the_same_commit_through_the_same_lookup_without_duplicating_it() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let repo = a_git_repo_with_one_commit("Add the terminal dock button");
    let project_id = app.a_project_in(&acme, "vega", repo.path()).await;
    let task_id = app.a_task(&project_id, "report-builder").await;

    app.call_tool("rekall_record_commit", json!({ "anchors": "project:vega task:report-builder" })).await;
    let pressed = app.post_empty(&format!("/api/tasks/{task_id}/commit-references/latest")).await;

    assert_eq!(pressed.status, 200);
    assert_eq!(pressed.str("comment"), "Add the terminal dock button");
    assert_eq!(
        app.count("SELECT COUNT(*) FROM commit_reference WHERE task_id = ?", vec![uuid_value(&task_id)]).await,
        1,
        "logging it twice, once from Claude and once from the button, is a no-op"
    );
}

#[tokio::test]
async fn a_commit_can_be_logged_against_a_step_separately_from_the_task_itself() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let repo = a_git_repo_with_one_commit("Add the CommitReference entity and migration");
    let project_id = app.a_project_in(&acme, "vega", repo.path()).await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    app.a_step(&task_id, "Backend: CommitReference entity", None).await;

    app.call_tool("rekall_record_commit", json!({ "anchors": "project:vega task:report-builder", "step": "1" })).await;

    assert_eq!(
        app.count("SELECT COUNT(*) FROM commit_reference WHERE task_id = ? AND step_id IS NOT NULL", vec![uuid_value(&task_id)]).await,
        1
    );
    assert_eq!(
        app.count("SELECT COUNT(*) FROM commit_reference WHERE task_id = ? AND step_id IS NULL", vec![uuid_value(&task_id)]).await,
        0,
        "the same commit is not also logged at the task level"
    );
}

#[tokio::test]
async fn auto_commit_can_only_be_switched_on_for_a_folder_that_is_a_git_repository() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let plain = tempfile::tempdir().unwrap();
    let repo = a_git_repo_with_one_commit("seed");
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;

    let on_plain = update_project(&app, &project_id, &acme, plain.path().to_str().unwrap(), true).await;
    assert_eq!(on_plain["autoCommit"], false);
    let plain_status = app.get(&format!("/api/projects/{project_id}/repository")).await.body;
    assert_eq!(plain_status["exists"], true);
    assert_eq!(plain_status["repository"], false);
    assert_eq!(plain_status["autoCommit"], false);

    let on_repo = update_project(&app, &project_id, &acme, repo.path().to_str().unwrap(), true).await;
    assert_eq!(on_repo["autoCommit"], true);
    let repo_status = app.get(&format!("/api/projects/{project_id}/repository")).await.body;
    assert_eq!(repo_status["repository"], true);
    assert_eq!(repo_status["autoCommit"], true);
    assert_eq!(repo_status["userEmail"], "test@example.com");
    assert!(!repo_status["branch"].is_null());
}

#[tokio::test]
async fn claiming_a_step_on_an_auto_commit_project_commits_the_folder_and_logs_it_against_the_step() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let repo = a_git_repo_with_one_commit("seed");
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    update_project(&app, &project_id, &acme, repo.path().to_str().unwrap(), true).await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    app.a_step(&task_id, "Wire the export endpoint", None).await;
    std::fs::write(repo.path().join("export.ts"), "export const x = 1\n").unwrap();

    let answer = app
        .call_tool("rekall_step", json!({ "anchors": "project:vega task:report-builder", "step": "1", "state": "claimed" }))
        .await;

    assert_contains!(answer, "Auto-committed", "feat: Wire the export endpoint", "\"Wire the export endpoint\"");
    assert_eq!(run_git(repo.path(), &["log", "-1", "--format=%s"]), "feat: Wire the export endpoint");
    assert_contains!(run_git(repo.path(), &["log", "-1", "--format=%b"]), "project:vega task:report-builder, step 1");
    assert_eq!(run_git(repo.path(), &["status", "--porcelain"]), "");
    assert_eq!(
        app.string("SELECT comment FROM commit_reference WHERE task_id = ? AND step_id IS NOT NULL", vec![uuid_value(&task_id)]).await,
        Some("feat: Wire the export endpoint".into())
    );
}

#[tokio::test]
async fn a_claim_with_a_clean_tree_commits_nothing_and_says_so_and_the_claim_stands() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let repo = a_git_repo_with_one_commit("seed");
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    update_project(&app, &project_id, &acme, repo.path().to_str().unwrap(), true).await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    app.a_step(&task_id, "Nothing changed", None).await;

    let answer = app
        .call_tool("rekall_step", json!({ "anchors": "project:vega task:report-builder", "step": "1", "state": "claimed" }))
        .await;

    assert_contains!(answer, "is now `claimed`", "nothing to commit");
    assert_eq!(run_git(repo.path(), &["log", "-1", "--format=%s"]), "seed");
    assert_eq!(app.count("SELECT COUNT(*) FROM commit_reference WHERE task_id = ?", vec![uuid_value(&task_id)]).await, 0);
}

#[tokio::test]
async fn the_wrapup_of_a_stepless_task_on_an_auto_commit_project_commits_against_the_task() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let repo = a_git_repo_with_one_commit("seed");
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    update_project(&app, &project_id, &acme, repo.path().to_str().unwrap(), true).await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    std::fs::write(repo.path().join("README.md"), "seed and more").unwrap();

    let answer = app
        .call_tool("rekall_wrapup", json!({ "anchors": "project:vega task:report-builder", "body": "## State\n\nDone." }))
        .await;

    assert_contains!(answer, "Wrapup written", "Auto-committed", "against the task");
    assert_eq!(run_git(repo.path(), &["log", "-1", "--format=%s"]), "docs: report-builder");
    assert_eq!(
        app.count("SELECT COUNT(*) FROM commit_reference WHERE task_id = ? AND step_id IS NULL", vec![uuid_value(&task_id)]).await,
        1
    );
}

#[tokio::test]
async fn the_wrapup_of_a_task_with_a_checklist_never_commits_its_steps_carry_the_claims() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let repo = a_git_repo_with_one_commit("seed");
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    update_project(&app, &project_id, &acme, repo.path().to_str().unwrap(), true).await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    app.a_step(&task_id, "Step one", None).await;
    std::fs::write(repo.path().join("README.md"), "seed and more").unwrap();

    let answer = app
        .call_tool("rekall_wrapup", json!({ "anchors": "project:vega task:report-builder", "body": "## State\n\nHalf done." }))
        .await;

    assert_lacks!(answer, "Auto-commit");
    assert_eq!(run_git(repo.path(), &["log", "-1", "--format=%s"]), "seed");
    assert!(!run_git(repo.path(), &["status", "--porcelain"]).is_empty());
}

#[tokio::test]
async fn a_project_that_does_not_auto_commit_leaves_the_tree_alone_on_a_claim() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let repo = a_git_repo_with_one_commit("seed");
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    update_project(&app, &project_id, &acme, repo.path().to_str().unwrap(), false).await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    app.a_step(&task_id, "Step one", None).await;
    std::fs::write(repo.path().join("export.ts"), "export const x = 1\n").unwrap();

    let answer = app
        .call_tool("rekall_step", json!({ "anchors": "project:vega task:report-builder", "step": "1", "state": "claimed" }))
        .await;

    assert_contains!(answer, "is now `claimed`");
    assert_lacks!(answer, "Auto-commit");
    assert_contains!(run_git(repo.path(), &["status", "--porcelain"]), "export.ts");
}

#[tokio::test]
async fn the_context_tells_a_session_when_its_project_auto_commits() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let repo = a_git_repo_with_one_commit("seed");
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    app.a_task(&project_id, "report-builder").await;

    let before = app.call_tool("rekall_context", json!({ "anchors": "project:vega task:report-builder" })).await;
    update_project(&app, &project_id, &acme, repo.path().to_str().unwrap(), true).await;
    let after = app.call_tool("rekall_context", json!({ "anchors": "project:vega task:report-builder" })).await;

    assert_lacks!(before, "auto-commit");
    assert_contains!(after, "`auto-commit`: on", "Do not `git commit` yourself");
}

#[tokio::test]
async fn the_diff_introduced_by_a_logged_commit_can_be_fetched_by_the_rows_own_id() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let repo = a_git_repo_with_one_commit("Add the README");
    let project_id = app.a_project_in(&acme, "vega", repo.path()).await;
    let task_id = app.a_task(&project_id, "report-builder").await;

    let logged = app.post_empty(&format!("/api/tasks/{task_id}/commit-references/latest")).await;
    let diff = app.get(&format!("/api/commit-references/{}/diff", logged.str("id"))).await;

    assert_eq!(diff.status, 200);
    assert_contains!(diff.str("diff"), "+Add the README");
}

#[tokio::test]
async fn the_diff_of_a_commit_reference_that_does_not_exist_is_a_404_not_an_empty_body() {
    let app = app().await;
    let response = app.get(&format!("/api/commit-references/{}/diff", rekall_common::Id::random())).await;
    assert_eq!(response.status, 404);
}

#[tokio::test]
async fn a_logged_commit_can_be_deleted_removing_its_row_entirely() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let repo = a_git_repo_with_one_commit("Add the delete button");
    let project_id = app.a_project_in(&acme, "vega", repo.path()).await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    let logged = app.post_empty(&format!("/api/tasks/{task_id}/commit-references/latest")).await;
    let reference_id = logged.str("id");

    let response = app.delete(&format!("/api/commit-references/{reference_id}")).await;

    assert_eq!(response.status, 200);
    assert_eq!(app.count("SELECT COUNT(*) FROM commit_reference WHERE id = ?", vec![uuid_value(&reference_id)]).await, 0);
}

#[tokio::test]
async fn deleting_a_commit_reference_that_does_not_exist_is_a_404_not_a_silent_no_op() {
    let app = app().await;
    let response = app.delete(&format!("/api/commit-references/{}", rekall_common::Id::random())).await;
    assert_eq!(response.status, 404);
}

#[tokio::test]
async fn the_recent_log_of_a_tasks_project_folder_is_listed_newest_first_for_picking_a_commit_by_hand() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let repo = a_git_repo_with_commits(&["Add the readme", "Add the picker", "Polish the picker"]);
    let project_id = app.a_project_in(&acme, "vega", repo.path()).await;
    let task_id = app.a_task(&project_id, "report-builder").await;

    let recent = app.get(&format!("/api/tasks/{task_id}/recent-commits")).await;

    assert_eq!(recent.status, 200);
    let entries = recent.list();
    let subjects: Vec<&str> = entries.iter().map(|e| e["subject"].as_str().unwrap()).collect();
    assert_eq!(subjects, ["Polish the picker", "Add the picker", "Add the readme"]);
    for entry in &entries {
        assert_eq!(entry["hash"].as_str().unwrap().len(), 40);
        assert!(!entry["committedAt"].is_null());
    }
}

#[tokio::test]
async fn a_past_commit_picked_by_its_hash_is_logged_with_its_own_subject_and_diff_not_the_tips() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let repo = a_git_repo_with_commits(&["Add the readme", "Add the picker", "Polish the picker"]);
    let project_id = app.a_project_in(&acme, "vega", repo.path()).await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    let middle = run_git(repo.path(), &["rev-parse", "HEAD~1"]);

    let logged = app.post(&format!("/api/tasks/{task_id}/commit-references"), json!({ "commitHash": &middle[..7] })).await;

    assert_eq!(logged.status, 200);
    assert_eq!(logged.str("commitHash"), middle);
    assert_eq!(logged.str("comment"), "Add the picker");
    let diff = app.get(&format!("/api/commit-references/{}/diff", logged.str("id"))).await;
    assert_contains!(diff.str("diff"), "+Add the picker");
    assert_lacks!(diff.str("diff"), "Polish");
}

#[tokio::test]
async fn a_past_commit_can_be_logged_against_a_step_by_its_hash() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let repo = a_git_repo_with_commits(&["Add the readme", "Add the picker"]);
    let project_id = app.a_project_in(&acme, "vega", repo.path()).await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    let step_id = app.a_step(&task_id, "Backend: picker endpoint", None).await;
    let first = run_git(repo.path(), &["rev-parse", "HEAD~1"]);

    let logged = app
        .post(&format!("/api/tasks/{task_id}/commit-references"), json!({ "commitHash": first, "stepId": step_id }))
        .await;

    assert_eq!(logged.str("stepId"), step_id);
    assert_eq!(logged.str("comment"), "Add the readme");
}

#[tokio::test]
async fn a_hash_that_names_no_commit_is_a_400_with_the_hash_in_the_reason_and_nothing_is_logged() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let repo = a_git_repo_with_one_commit("Add the readme");
    let project_id = app.a_project_in(&acme, "vega", repo.path()).await;
    let task_id = app.a_task(&project_id, "report-builder").await;

    let refused = app.post(&format!("/api/tasks/{task_id}/commit-references"), json!({ "commitHash": "deadbeef" })).await;

    assert_eq!(refused.status, 400);
    assert_contains!(refused.detail(), "deadbeef");
    assert_eq!(app.count("SELECT COUNT(*) FROM commit_reference WHERE task_id = ?", vec![uuid_value(&task_id)]).await, 0);
}

#[tokio::test]
async fn rekall_record_commit_logs_an_earlier_commit_when_a_session_names_its_hash() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let repo = a_git_repo_with_commits(&["Wire up the ledger", "Fix the ledger"]);
    let project_id = app.a_project_in(&acme, "vega", repo.path()).await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    let first = run_git(repo.path(), &["rev-parse", "HEAD~1"]);

    let logged = app
        .call_tool("rekall_record_commit", json!({ "anchors": "project:vega task:report-builder", "commit": &first[..8] }))
        .await;

    assert_contains!(logged, "Wire up the ledger");
    assert_eq!(
        app.string("SELECT commit_hash FROM commit_reference WHERE task_id = ?", vec![uuid_value(&task_id)]).await,
        Some(first)
    );
}

#[tokio::test]
async fn a_logged_commit_is_off_the_context_until_the_console_chooses_it_then_arrives_with_its_diff() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let repo = a_git_repo_with_one_commit("Wire up the context toggle");
    let project_id = app.a_project_in(&acme, "vega", repo.path()).await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    let logged = app.post_empty(&format!("/api/tasks/{task_id}/commit-references/latest")).await;
    let reference_id = logged.str("id");
    let hash = logged.str("commitHash");
    assert_eq!(logged.get("inContext"), &json!(false));

    let context = app.call_tool("rekall_context", json!({ "anchors": "project:vega task:report-builder" })).await;
    assert_lacks!(context, "<commits", &hash);

    let chosen = app.patch(&format!("/api/commit-references/{reference_id}"), json!({ "inContext": true })).await;
    assert_eq!(chosen.status, 200);
    assert_eq!(chosen.get("inContext"), &json!(true));

    let context = app.call_tool("rekall_context", json!({ "anchors": "project:vega task:report-builder" })).await;
    assert_contains!(
        context,
        "<commits count=\"1\">",
        &format!("<commit hash=\"{hash}\">"),
        "Wire up the context toggle",
        "+Wire up the context toggle",
        "</commits>"
    );

    app.patch(&format!("/api/commit-references/{reference_id}"), json!({ "inContext": false })).await;
    assert_lacks!(app.call_tool("rekall_context", json!({ "anchors": "project:vega task:report-builder" })).await, "<commits");
}

#[tokio::test]
async fn a_commit_chosen_on_a_step_names_that_step_in_the_context_so_the_session_knows_what_it_belongs_to() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let repo = a_git_repo_with_one_commit("Wire the ledger");
    let project_id = app.a_project_in(&acme, "vega", repo.path()).await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    let step_id = support::id(&app.post(&format!("/api/tasks/{task_id}/steps"), json!({ "title": "Wire the ledger" })).await);
    let logged = app
        .post(
            &format!("/api/tasks/{task_id}/commit-references"),
            json!({ "commitHash": run_git(repo.path(), &["rev-parse", "HEAD"]), "stepId": step_id }),
        )
        .await;

    app.patch(&format!("/api/commit-references/{}", logged.str("id")), json!({ "inContext": true })).await;

    assert_contains!(
        app.call_tool("rekall_context", json!({ "anchors": "project:vega task:report-builder" })).await,
        "step=\"Wire the ledger\">"
    );
}

#[tokio::test]
async fn choosing_a_commit_reference_that_does_not_exist_for_the_context_is_a_404() {
    let app = app().await;
    let response = app.patch(&format!("/api/commit-references/{}", rekall_common::Id::random()), json!({ "inContext": true })).await;
    assert_eq!(response.status, 404);
}
