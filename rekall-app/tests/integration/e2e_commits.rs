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

    let logged = app.call_tool("rekall_record_commit", json!({ "anchors": "task:VE-1" })).await;

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

    app.call_tool("rekall_record_commit", json!({ "anchors": "task:VE-1" })).await;
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

    app.call_tool("rekall_record_commit", json!({ "anchors": "task:VE-1", "step": "1" })).await;

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
        .call_tool("rekall_step", json!({ "anchors": "task:VE-1", "step": "1", "state": "claimed" }))
        .await;

    assert_contains!(answer, "Auto-committed", "feat: Wire the export endpoint", "\"Wire the export endpoint\"");
    assert_eq!(run_git(repo.path(), &["log", "-1", "--format=%s"]), "feat: Wire the export endpoint");
    assert_contains!(run_git(repo.path(), &["log", "-1", "--format=%b"]), "task:VE-1, step 1");
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
        .call_tool("rekall_step", json!({ "anchors": "task:VE-1", "step": "1", "state": "claimed" }))
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
        .call_tool("rekall_wrapup", json!({ "anchors": "task:VE-1", "body": "## State\n\nDone." }))
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
        .call_tool("rekall_wrapup", json!({ "anchors": "task:VE-1", "body": "## State\n\nHalf done." }))
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
        .call_tool("rekall_step", json!({ "anchors": "task:VE-1", "step": "1", "state": "claimed" }))
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

    let before = app.call_tool("rekall_context", json!({ "anchors": "task:VE-1" })).await;
    update_project(&app, &project_id, &acme, repo.path().to_str().unwrap(), true).await;
    let after = app.call_tool("rekall_context", json!({ "anchors": "task:VE-1" })).await;

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
async fn a_tasks_changes_list_its_logged_commits_with_files_and_hunks() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let repo = a_git_repo_with_one_commit("Add the README");
    let project_id = app.a_project_in(&acme, "vega", repo.path()).await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    app.post_empty(&format!("/api/tasks/{task_id}/commit-references/latest")).await;

    let changes = app.get(&format!("/api/tasks/{task_id}/changes")).await;

    assert_eq!(changes.status, 200);
    let repository = &changes.body["repositories"][0];
    let commit = &repository["commits"][0];
    assert_eq!(repository["componentId"], Value::Null);
    assert_eq!(commit["subject"], "Add the README");
    assert_eq!(commit["available"], true);
    assert_eq!(commit["files"][0]["kind"], "ADDED");
    assert_eq!(commit["files"][0]["hunks"][0]["lines"][0]["kind"], "ADDED");
    assert_eq!(repository["workingTree"], Value::Null);
}

#[tokio::test]
async fn a_clean_task_has_an_index_without_change_sources() {
    let app = app().await;
    let company = app.a_company("Acme").await;
    let repo = a_git_repo_with_one_commit("Seed");
    let project_id = app.a_project_in(&company, "vega", repo.path()).await;
    let task_id = app.a_task(&project_id, "report-builder").await;

    let index = app.get(&format!("/api/tasks/{task_id}/changes/index")).await;

    assert_eq!(index.status, 200);
    assert!(index.str("snapshotId").parse::<rekall_kernel::Id>().is_ok());
    assert_eq!(index.body["repositories"], json!([]));
}

#[tokio::test]
async fn a_changes_index_lists_all_files_and_batches_return_only_the_requested_sources() {
    let app = app().await;
    let company = app.a_company("Acme").await;
    let project_repo = a_git_repo_with_one_commit("Seed project");
    let component_repo = a_git_repo_with_one_commit("Seed component");
    let project_id = app.a_project_in(&company, "vega", project_repo.path()).await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    let component = app.post(&format!("/api/projects/{project_id}/components"), json!({
        "name": "Library",
        "folder": component_repo.path(),
    })).await;
    assert_eq!(component.status, 201, "{}", component.detail());
    let component_id = component.str("id");
    let project_commit = log_changed_files(&app, &task_id, project_repo.path(), None, 16).await;
    let component_commit = log_changed_files(&app, &task_id, component_repo.path(), Some(&component_id), 9).await;
    std::fs::write(project_repo.path().join("file-09.txt"), "pending edit\n").unwrap();
    let base = format!("/api/tasks/{task_id}/changes");

    let index = app.get(&format!("{base}/index")).await;

    assert_eq!(index.status, 200);
    let project_files = index.body["repositories"][0]["commits"][0]["files"].as_array().unwrap();
    let component_files = index.body["repositories"][1]["commits"][0]["files"].as_array().unwrap();
    assert_eq!((project_files.len(), component_files.len()), (16, 9));
    assert_eq!(index.body["repositories"][1]["componentId"], component_id);
    assert_eq!(index.body["repositories"][0]["workingTree"]["files"][0]["path"], "file-09.txt");
    assert_lacks!(index.text, "\"hunks\"", "\"lines\"");

    let snapshot = index.get("snapshotId");
    let project_source = json!({ "kind": "commit", "referenceId": project_commit });
    let batches = [
        (1..=8, None, project_source.clone()),
        (9..=16, None, project_source),
        (5..=9, Some(component_id.as_str()), json!({ "kind": "commit", "referenceId": component_commit })),
    ];
    for (numbers, component, source) in batches {
        let keys: Vec<_> = numbers.map(|number| changes_file_key(component, &format!("file-{number:02}.txt"))).collect();
        let response = app.post(&format!("{base}/files"), changes_batch(snapshot, &keys)).await;
        assert_eq!(response.status, 200, "{}", response.detail());
        let entries = response.list();
        let committed: Vec<_> = entries.iter().filter(|entry| entry["source"] == source).map(|entry| entry["file"].clone()).collect();
        assert_eq!(committed, keys);
        let includes_edit = component.is_none() && keys.contains(&changes_file_key(None, "file-09.txt"));
        assert_eq!(entries.len(), keys.len() + usize::from(includes_edit));
        assert_eq!(entries[0]["hunks"][0]["lines"][0]["newNumber"], 1);
        if includes_edit {
            let entry = entries.last().unwrap();
            assert_eq!(entry["source"], json!({ "kind": "working-tree" }));
            assert_eq!(entry["file"], changes_file_key(None, "file-09.txt"));
            assert_eq!(entry["hunks"][0]["lines"][1]["text"], "pending edit");
        }
    }

    let scoped = app.post(&format!("{base}/files"), json!({
        "snapshotId": snapshot,
        "scope": { "kind": "working-tree", "componentId": null },
        "files": [changes_file_key(None, "file-09.txt")],
    })).await;
    assert_eq!(scoped.status, 200, "{}", scoped.detail());
    let entries = scoped.list();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["source"], json!({ "kind": "working-tree" }));
}

#[tokio::test]
async fn changes_batches_refuse_invalid_keys_and_snapshots_with_distinct_statuses() {
    let app = app().await;
    let company = app.a_company("Acme").await;
    let project_id = app.a_project(&company, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    let other_task = app.a_task(&project_id, "other-report").await;
    let base = format!("/api/tasks/{task_id}/changes");
    let files = format!("{base}/files");
    let index = app.get(&format!("{base}/index")).await;
    let snapshot = index.get("snapshotId");
    let key = changes_file_key(None, "README.md");
    let one_file = std::slice::from_ref(&key);
    let too_many: Vec<_> = (1..=9).map(|number| changes_file_key(None, &format!("file-{number}.txt"))).collect();
    let invalid = [
        (changes_batch(snapshot, &[]), "between 1 and 8"),
        (changes_batch(snapshot, &too_many), "between 1 and 8"),
        (changes_batch(snapshot, &[key.clone(), key.clone()]), "distinct"),
        (changes_batch(&json!("invalid"), one_file), "invalid"),
        (changes_batch(snapshot, &[json!({ "path": "README.md" })]), "componentId"),
        (changes_batch(snapshot, &[changes_file_key(None, "")]), "non-empty paths"),
        (json!({ "snapshotId": snapshot, "scope": { "kind": "invalid" }, "files": one_file }), "invalid"),
        (changes_batch(snapshot, one_file), "not indexed"),
    ];
    for (request, detail) in invalid {
        let response = app.post(&files, request).await;
        assert_eq!(response.status, 400, "{}", response.detail());
        assert_contains!(response.detail(), detail);
    }

    let cross_task = app.post(&format!("/api/tasks/{other_task}/changes/files"), changes_batch(snapshot, one_file)).await;
    assert_eq!(cross_task.status, 404);

    for _ in 0..16 {
        assert_eq!(app.get(&format!("{base}/index")).await.status, 200);
    }
    let expired = app.post(&files, changes_batch(snapshot, one_file)).await;
    assert_eq!(expired.status, 409);
    assert_contains!(expired.detail(), "snapshot has expired");
}

async fn log_changed_files(app: &App, task_id: &str, repo: &std::path::Path, component: Option<&str>, count: usize) -> String {
    for number in 1..=count {
        std::fs::write(repo.join(format!("file-{number:02}.txt")), "committed line\n").unwrap();
    }
    run_git(repo, &["add", "."]);
    run_git(repo, &["commit", "-qm", "Add review files"]);
    let commit_hash = run_git(repo, &["rev-parse", "HEAD"]);
    let logged = app.post(&format!("/api/tasks/{task_id}/commit-references"), json!({
        "commitHash": commit_hash,
        "componentId": component,
    })).await;
    assert_eq!(logged.status, 200, "{}", logged.detail());
    logged.str("id")
}

fn changes_file_key(component: Option<&str>, path: &str) -> Value {
    json!({ "componentId": component, "path": path })
}

fn changes_batch(snapshot: &Value, files: &[Value]) -> Value {
    json!({ "snapshotId": snapshot, "scope": { "kind": "all" }, "files": files })
}

#[tokio::test]
async fn the_diff_of_a_commit_reference_that_does_not_exist_is_a_404_not_an_empty_body() {
    let app = app().await;
    let response = app.get(&format!("/api/commit-references/{}/diff", rekall_kernel::Id::random())).await;
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
    let response = app.delete(&format!("/api/commit-references/{}", rekall_kernel::Id::random())).await;
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
        .call_tool("rekall_record_commit", json!({ "anchors": "task:VE-1", "commit": &first[..8] }))
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

    let context = app.call_tool("rekall_context", json!({ "anchors": "task:VE-1" })).await;
    assert_lacks!(context, "<commits", &hash);

    let chosen = app.patch(&format!("/api/commit-references/{reference_id}"), json!({ "inContext": true })).await;
    assert_eq!(chosen.status, 200);
    assert_eq!(chosen.get("inContext"), &json!(true));

    let context = app.call_tool("rekall_context", json!({ "anchors": "task:VE-1" })).await;
    assert_contains!(
        context,
        "<commits count=\"1\">",
        &format!("<commit hash=\"{hash}\">"),
        "Wire up the context toggle",
        "+Wire up the context toggle",
        "</commits>"
    );

    app.patch(&format!("/api/commit-references/{reference_id}"), json!({ "inContext": false })).await;
    assert_lacks!(app.call_tool("rekall_context", json!({ "anchors": "task:VE-1" })).await, "<commits");
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
        app.call_tool("rekall_context", json!({ "anchors": "task:VE-1" })).await,
        "step=\"Wire the ledger\">"
    );
}

#[tokio::test]
async fn choosing_a_commit_reference_that_does_not_exist_for_the_context_is_a_404() {
    let app = app().await;
    let response = app.patch(&format!("/api/commit-references/{}", rekall_kernel::Id::random()), json!({ "inContext": true })).await;
    assert_eq!(response.status, 404);
}
