//! `RekallEndToEndTest`, part five: companies, proposed steps, notes a
//! session writes, the path picker's folder listing, search, context size, the stateless MCP era over real HTTP, the console's routes, and local access.

mod support;

use std::sync::Arc;

use async_trait::async_trait;
use rekall_core::version::{Release, ReleaseFeed, ReleaseFeedError};
use rekall_app::StartOptions;
use serde_json::json;
use support::{app, app_with, id, App};

#[tokio::test]
async fn a_project_label_two_companies_share_is_disambiguated_by_the_company_anchor() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let globex = app.a_company("Globex").await;
    app.a_project(&acme, "website", "ACTIVE").await;
    app.a_project(&globex, "website", "PAUSED").await;

    assert_contains!(
        app.call_tool("rekall_context", json!({ "anchors": "project:website" })).await,
        "matches 2 records",
        "company:Acme",
        "company:Globex"
    );
}

#[tokio::test]
async fn a_company_anchor_lists_its_projects_and_a_task_reaches_back_up_to_it() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let vega = app.a_project(&acme, "vega", "ACTIVE").await;
    app.a_task(&vega, "report-builder").await;

    assert_contains!(app.call_tool("rekall_context", json!({ "anchors": "company:Acme" })).await, "Company: Acme", "`project:vega`");
    assert_contains!(
        app.call_tool("rekall_context", json!({ "anchors": "task:report-builder" })).await,
        "Task: report-builder",
        "Project: vega",
        "Company: Acme"
    );
}

#[tokio::test]
async fn deleting_a_company_takes_its_projects_and_tasks_and_sweeps_the_orphaned_notes() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let vega = app.a_project(&acme, "vega", "ACTIVE").await;
    let task = app.a_task(&vega, "t").await;
    app.post("/api/documents", json!({ "title": "n.md", "kind": "notes", "taskIds": [task], "bodyMarkdown": "x" })).await;

    app.delete(&format!("/api/companies/{acme}")).await;

    assert_eq!(app.count("SELECT COUNT(*) FROM project", vec![]).await, 0);
    assert_eq!(app.count("SELECT COUNT(*) FROM task", vec![]).await, 0);
    assert_eq!(app.count("SELECT COUNT(*) FROM document", vec![]).await, 0, "the notes had nothing left to hang from");
}

#[tokio::test]
async fn a_project_cannot_exist_without_a_company() {
    let app = app().await;
    let refused = app.post("/api/projects", json!({ "label": "orphan", "title": "Orphan", "status": "ACTIVE" })).await;
    assert_eq!(refused.status, 404);
}

// --- Proposed steps

#[tokio::test]
async fn a_session_can_propose_a_step_which_lands_as_a_draft_no_session_reads_and_the_same_title_twice_is_refused() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;

    let answer = app
        .call_tool(
            "rekall_propose_step",
            json!({ "anchors": "project:vega task:report-builder", "title": "Expose the report as a download", "detail": "Stream it; do not buffer." }),
        )
        .await;

    assert_contains!(answer, "Draft \"Expose the report as a download\" proposed", "1 draft");
    assert_eq!(app.get(&format!("/api/tasks/{task_id}/steps")).await.list()[0]["state"], "DRAFT");
    assert_lacks!(
        app.call_tool("rekall_context", json!({ "anchors": "project:vega task:report-builder" })).await,
        "Expose the report as a download"
    );
    assert_contains!(
        app.call_tool(
            "rekall_propose_step",
            json!({ "anchors": "project:vega task:report-builder", "title": "expose the report as a download" })
        )
        .await,
        "already has a step titled"
    );
    assert_contains!(
        app.call_tool("rekall_step", json!({ "anchors": "project:vega task:report-builder", "step": "1", "state": "running" })).await,
        "still a draft"
    );
}

// --- Tasks created by a session

#[tokio::test]
async fn a_session_creates_a_task_whose_steps_are_drafts_and_the_console_lists_it() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;

    let answer = app
        .call_tool(
            "rekall_create_task",
            json!({
                "anchors": "project:vega",
                "title": "Report builder",
                "description": "Let people export a report.",
                "steps": [{ "title": "Expose the report as a download", "detail": "Stream it." }, { "title": "Add the button" }]
            }),
        )
        .await;

    assert_contains!(answer, "Task \"Report builder\" created as `project:vega task:report-builder`", "2 draft steps");
    let tasks = app.get(&format!("/api/tasks?projectId={project_id}")).await.list();
    assert_eq!(tasks[0]["description"], "Let people export a report.");
    let steps = app.get(&format!("/api/tasks/{}/steps", tasks[0]["id"].as_str().unwrap())).await.list();
    assert_eq!(steps.len(), 2);
    assert!(steps.iter().all(|step| step["state"] == "DRAFT"));
    assert_contains!(
        app.call_tool("rekall_create_task", json!({ "anchors": "project:vega", "title": "Report builder" })).await,
        "already uses the label 'report-builder'"
    );
}

#[tokio::test]
async fn a_session_creates_an_epic_and_files_tasks_under_it_and_the_console_lists_them() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    app.a_task(&project_id, "login").await;

    let epic = app.call_tool("rekall_create_task", json!({ "anchors": "project:vega", "title": "Checkout", "kind": "epic" })).await;
    app.call_tool("rekall_create_task", json!({ "anchors": "project:vega", "title": "Pay", "epic": "checkout" })).await;
    let joined = app.call_tool("rekall_join_epic", json!({ "anchors": "project:vega task:login", "epic": "checkout" })).await;

    assert_contains!(epic, "Epic \"Checkout\" created as `project:vega task:checkout`");
    assert_contains!(joined, "`project:vega task:login` is filed under the epic \"Checkout\"");
    let tasks = app.get(&format!("/api/tasks?projectId={project_id}")).await.list();
    let checkout = tasks.iter().find(|task| task["label"] == "checkout").unwrap();
    assert_eq!(checkout["kind"], "EPIC");
    for label in ["login", "pay"] {
        let task = tasks.iter().find(|task| task["label"] == label).unwrap();
        assert_eq!(task["epicId"], checkout["id"], "{label}");
    }
    assert_contains!(
        app.call_tool("rekall_create_task", json!({ "anchors": "project:vega", "title": "Billing", "kind": "epic", "steps": [{ "title": "One" }] })).await,
        "An epic holds no steps"
    );
}

// --- Sections read by a session

#[tokio::test]
async fn a_session_reads_the_sections_of_a_project_listed_by_title_or_one_whole_while_no_context_carries_them() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    app.a_task(&project_id, "report-builder").await;
    let anchors = "project:vega task:report-builder";
    assert_contains!(app.call_tool("rekall_read_sections", json!({ "anchors": "project:vega" })).await, "has no sections");
    app.call_tool("rekall_project_section", json!({ "anchors": anchors, "title": "Deployment", "body": "Push a tag." })).await;

    let listed = app.call_tool("rekall_read_sections", json!({ "anchors": anchors })).await;
    let one = app.call_tool("rekall_read_sections", json!({ "anchors": "project:vega", "title": "deployment" })).await;

    assert_contains!(listed, "1 section", "- Deployment (11 characters)");
    assert_lacks!(listed, "Push a tag.");
    assert_contains!(one, "Section \"Deployment\" of `project:vega`", "Push a tag.");
    assert_lacks!(app.call_tool("rekall_context", json!({ "anchors": anchors })).await, "Push a tag.");
    assert_contains!(
        app.call_tool("rekall_read_sections", json!({ "anchors": "project:vega", "title": "Glossary" })).await,
        "has no section titled 'Glossary'",
        "has: Deployment"
    );
}

// --- Notes written by a session

#[tokio::test]
async fn a_session_writes_a_note_onto_a_task_which_travels_with_its_context_and_a_title_the_tasks_notes_carry_is_refused() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;

    let answer = app
        .call_tool(
            "rekall_note",
            json!({ "anchors": "project:vega task:report-builder", "title": "export-formats.md", "body": "CSV and XLSX, streamed." }),
        )
        .await;

    assert_contains!(answer, "Note \"export-formats.md\" written on `project:vega task:report-builder`", "1 note,");
    assert!(regex::Regex::new("as `note:[0-9a-f]{8}`").unwrap().is_match(&answer), "{answer}");
    let notes = app.documents_on(&task_id).await;
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0]["kind"], "notes");
    assert_contains!(
        app.call_tool("rekall_context", json!({ "anchors": "project:vega task:report-builder" })).await,
        "export-formats.md",
        "CSV and XLSX, streamed."
    );

    // Reusing a title without `replace` is refused rather than overwriting.
    assert_contains!(
        app.call_tool(
            "rekall_note",
            json!({ "anchors": "project:vega task:report-builder", "title": "Export-Formats.md", "body": "Something else." })
        )
        .await,
        "already has a note titled"
    );
    // A note belongs to exactly one task when a session writes it.
    assert_contains!(
        app.call_tool("rekall_note", json!({ "anchors": "project:vega", "title": "x.md", "body": "y" })).await,
        "No task in those anchors"
    );
    assert_contains!(
        app.call_tool("rekall_note", json!({ "anchors": "project:vega task:report-builder", "title": "x.md", "body": "  " }))
            .await,
        "'body' is required"
    );
    assert_eq!(app.documents_on(&task_id).await.len(), 1);
}

#[tokio::test]
async fn a_session_rewrites_a_note_by_its_title_when_asked_to_replace_and_every_task_it_is_on_reads_the_new_body() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    let other_id = app.a_task(&project_id, "exporter").await;
    let document_id = app
        .post(
            "/api/documents",
            json!({ "title": "Results.md", "kind": "notes", "taskIds": [task_id, other_id], "bodyMarkdown": "| check | outcome |" }),
        )
        .await
        .str("id")
        .to_string();

    let answer = app
        .call_tool(
            "rekall_note",
            json!({ "anchors": "project:vega task:report-builder", "title": "results.md", "body": "All green.", "replace": true }),
        )
        .await;

    assert_contains!(answer, "Note \"Results.md\" on `project:vega task:report-builder` rewritten", "1 other task,");
    let notes = app.documents_on(&task_id).await;
    assert_eq!(notes.len(), 1, "rewritten in place, not added");
    assert_eq!(notes[0]["id"], document_id.as_str());
    assert_eq!(notes[0]["title"], "Results.md", "the title keeps the console's spelling");
    assert_eq!(notes[0]["bodyMarkdown"], "All green.");
    assert_eq!(app.documents_on(&other_id).await[0]["bodyMarkdown"], "All green.");

    // With nothing to replace, `replace` adds the note.
    assert_contains!(
        app.call_tool(
            "rekall_note",
            json!({ "anchors": "project:vega task:report-builder", "title": "runbook.md", "body": "Step one.", "replace": "true" })
        )
        .await,
        "written on `project:vega task:report-builder`",
        "2 notes,"
    );
}

// --- The path picker's folder listing

#[tokio::test]
async fn the_path_picker_lists_a_folder_from_the_home_folder_down_and_refuses_what_is_not_one() {
    let app = app().await;
    let home = app.folder.path().join("home");
    std::fs::create_dir_all(home.join("project/src")).unwrap();
    std::fs::write(home.join("project/README.md"), "#").unwrap();

    let at_home = app.get("/api/filesystem/directory").await;
    assert_eq!(at_home.status, 200);
    assert_eq!(at_home.str("home"), home.to_string_lossy());
    assert_eq!(at_home.str("path"), home.to_string_lossy());

    let base = home.join("project");
    let listing = app.get(&format!("/api/filesystem/directory?base={}", base.display())).await;
    assert_eq!(listing.status, 200);
    let names: Vec<&str> = listing.get("entries").as_array().unwrap().iter().map(|e| e["name"].as_str().unwrap()).collect();
    assert_eq!(names, ["src", "README.md"]);
    assert_eq!(listing.get("entries")[0]["directory"], true);
    assert_eq!(listing.get("readable"), true);
    assert_eq!(listing.get("truncated"), false);
    assert_eq!(listing.get("segments").as_array().unwrap().last().unwrap()["name"], "project");

    let typed = app.get(&format!("/api/filesystem/directory?base={}&path=~/project/src", base.display())).await;
    assert_eq!(typed.str("path"), home.join("project/src").to_string_lossy());
    assert_eq!(typed.str("parent"), base.to_string_lossy());

    assert_eq!(app.get(&format!("/api/filesystem/directory?base={}&path=nope", base.display())).await.status, 404);
    let file = app.get(&format!("/api/filesystem/directory?base={}&path=README.md", base.display())).await;
    assert_eq!(file.status, 400);
    assert_contains!(file.detail(), "is a file, not a folder.");
}

// --- Search

#[tokio::test]
async fn search_finds_a_phrase_in_descriptions_steps_wrapups_and_notes_with_the_words_around_it() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = id(&app
        .post(
            "/api/tasks",
            json!({ "label": "settlement", "title": "Settlement", "status": "TODO", "projectId": project_id, "description": "The nightly settlement batch reconciles the ledger." }),
        )
        .await);
    app.a_step(&task_id, "Wire the batch", Some("Schedule the settlement batch at 02:00.")).await;
    app.call_tool("rekall_wrapup", json!({ "anchors": "project:vega task:settlement", "body": "The settlement batch runs." })).await;
    app.post(
        "/api/documents",
        json!({ "title": "batch.md", "kind": "notes", "taskIds": [task_id], "bodyMarkdown": "Settlement batch logs live in /var/log/batch." }),
    )
    .await;

    let hits = app.get("/api/search?q=settlement%20batch").await.list();

    let kinds: Vec<&str> = hits.iter().map(|h| h["kind"].as_str().unwrap()).collect();
    assert_eq!(kinds, ["DESCRIPTION", "STEP", "WRAPUP", "NOTE"]);
    assert!(hits.iter().all(|h| h["excerpt"].as_str().unwrap().to_lowercase().contains("settlement batch")));
    assert!(app.get("/api/search?q=100%25_").await.list().is_empty(), "the LIKE wildcards are matched literally");
}

// --- Context size and reference notes

async fn context_size(app: &App, task_id: &str) -> i64 {
    app.get(&format!("/api/tasks/{task_id}/context-size")).await.get("characters").as_i64().unwrap()
}

#[tokio::test]
async fn a_note_sent_by_reference_travels_as_a_line_and_an_anchor_loads_in_full_by_that_anchor_and_weighs_less() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    let body = format!("Cluster kmaster14, accesso via bastion.\n\n{}", "Dettaglio lungo. ".repeat(400));
    let note = app
        .post("/api/documents", json!({ "title": "kmaster14.md", "kind": "notes", "taskIds": [task_id], "bodyMarkdown": body }))
        .await;
    let full_size = context_size(&app, &task_id).await;

    app.put(
        &format!("/api/documents/{}", note.str("id")),
        json!({ "title": "kmaster14.md", "kind": "notes", "taskIds": [task_id], "bodyMarkdown": body, "contextMode": "REFERENCE" }),
    )
    .await;

    let anchor = note.str("anchor");
    let context = app.call_tool("rekall_context", json!({ "anchors": "project:vega task:report-builder" })).await;
    assert_contains!(context, "loaded=\"on request\"", "Cluster kmaster14, accesso via bastion.", &anchor);
    assert_lacks!(context, "Dettaglio lungo.");
    assert!(context_size(&app, &task_id).await < full_size / 4, "the size the console shows drops with it");

    assert_contains!(app.call_tool("rekall_context", json!({ "anchors": anchor })).await, "Note: kmaster14.md", "Dettaglio lungo.");
    assert_contains!(app.call_tool("rekall_context", json!({ "anchors": "note:abc" })).await, "is not a note anchor");
}

#[tokio::test]
async fn the_context_size_is_what_rk_hands_over_split_into_parts_heaviest_first() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    app.call_tool("rekall_wrapup", json!({ "anchors": "project:vega task:report-builder", "body": "W".repeat(4000) })).await;

    let size = app.get(&format!("/api/tasks/{task_id}/context-size")).await;
    let context = app.call_tool("rekall_context", json!({ "anchors": "project:vega task:report-builder" })).await;
    let length = context.encode_utf16().count() as i64;

    assert_eq!(size.get("characters").as_i64().unwrap(), length);
    let parts = size.get("parts").as_array().unwrap().clone();
    assert_eq!(parts[0]["label"], "Wrapup");
    assert_eq!(parts.iter().map(|p| p["characters"].as_i64().unwrap()).sum::<i64>(), length);
}

// --- 2026-07-28 era over real HTTP: the only place that proves the mirrored headers are bound.

#[tokio::test]
async fn a_stateless_era_call_is_served_with_no_handshake_before_it() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    app.a_project(&acme, "vega", "ACTIVE").await;

    let response = app
        .modern_rpc(
            "tools/call",
            Some("rekall_context"),
            json!({ "name": "rekall_context", "arguments": { "anchors": "project:vega" } }),
        )
        .await;

    assert_eq!(response.status, 200);
    assert_eq!(response.body["result"]["isError"], json!(false));
    assert_contains!(response.body["result"]["content"][0]["text"].as_str().unwrap(), "project:vega");
}

#[tokio::test]
async fn a_stateless_era_call_whose_headers_disagree_with_its_body_is_refused() {
    let app = app().await;
    let response = app
        .modern_rpc(
            "tools/call",
            Some("something_else"),
            json!({ "name": "rekall_context", "arguments": { "anchors": "project:vega" } }),
        )
        .await;

    assert_eq!(response.status, 400);
    assert_eq!(response.body["error"]["code"], json!(-32020));
}

#[tokio::test]
async fn server_discover_names_the_revisions_this_server_speaks() {
    let app = app().await;
    let response = app.modern_rpc("server/discover", None, json!({})).await;

    assert_eq!(response.status, 200);
    let versions: Vec<&str> =
        response.body["result"]["supportedVersions"].as_array().unwrap().iter().map(|v| v.as_str().unwrap()).collect();
    assert!(versions.contains(&"2026-07-28"), "{versions:?}");
}

// Regression: returning the entity before flush left updatedAt null and the frontend rejected the create.
#[tokio::test]
async fn a_create_answers_with_the_timestamps_already_written_not_with_nulls() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project = app.post("/api/projects", json!({ "label": "vega", "title": "Vega", "status": "ACTIVE", "companyId": acme })).await;
    assert!(!project.get("updatedAt").is_null());
    let project_id = id(&project);

    let task = app
        .post("/api/tasks", json!({ "label": "report-builder", "title": "Report builder", "status": "TODO", "projectId": project_id }))
        .await;
    assert!(!task.get("updatedAt").is_null());

    let document = app
        .post("/api/documents", json!({ "title": "CONTEXT.md", "kind": "context", "taskIds": [id(&task)], "bodyMarkdown": "x" }))
        .await;
    assert!(!document.get("updatedAt").is_null());
}

#[tokio::test]
async fn a_status_outside_the_enum_is_the_callers_mistake_so_it_is_a_400_and_not_a_500() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;

    let refused = app.post("/api/tasks", json!({ "label": "t", "title": "T", "status": "OPEN", "projectId": project_id })).await;
    assert_eq!(refused.status, 400);
}

#[tokio::test]
async fn an_id_that_is_not_a_uuid_is_the_callers_mistake_so_it_is_a_400_and_not_a_500() {
    let app = app().await;

    let refused = app.get("/api/tasks/not-a-uuid").await;
    assert_eq!(refused.status, 400);
    assert_eq!(refused.body["detail"], "Parameter 'id' is not a UUID: not-a-uuid");
}

#[tokio::test]
async fn a_review_without_a_target_state_is_refused_as_a_400() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task = app.post("/api/tasks", json!({ "label": "t", "title": "T", "status": "TODO", "projectId": project_id })).await;

    let refused = app.patch(&format!("/api/tasks/{}/review", task.body["id"].as_str().unwrap()), json!({})).await;
    assert_eq!(refused.status, 400);
}

#[tokio::test]
async fn both_a_label_and_a_title_are_required() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;

    assert_eq!(app.post("/api/tasks", json!({ "label": "t", "status": "TODO", "projectId": project_id })).await.status, 400);
    assert_eq!(app.post("/api/tasks", json!({ "title": "T", "status": "TODO", "projectId": project_id })).await.status, 400);
}

#[tokio::test]
async fn every_task_unscoped_comes_back_grouped_by_project_and_then_by_label() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let vega = app.a_project(&acme, "vega", "ACTIVE").await;
    let beacon = app.a_project(&acme, "beacon", "ACTIVE").await;
    app.a_task(&vega, "setup").await;
    app.a_task(&vega, "report-builder").await;
    app.a_task(&beacon, "wiring").await;

    let tasks = app.get("/api/tasks").await.list();

    let order: Vec<String> =
        tasks.iter().map(|t| format!("{}/{}", t["projectLabel"].as_str().unwrap(), t["label"].as_str().unwrap())).collect();
    assert_eq!(order, ["beacon/wiring", "vega/report-builder", "vega/setup"]);
}

// Regression: a stale forwarding list left current routes answering 404 on a refresh.
#[tokio::test]
async fn a_refresh_on_any_ui_route_serves_the_application_and_an_unknown_api_path_still_fails() {
    let app = app().await;
    let random = rekall_kernel::Id::random();
    for path in [
        "/".to_string(),
        "/projects".into(),
        format!("/projects/{random}"),
        "/tasks".into(),
        "/report".into(),
        format!("/tasks/{random}"),
        "/search".into(),
        "/calendar".into(),
        "/diagrams".into(),
    ] {
        let response = app.get(&path).await;
        assert_eq!(response.status, 200, "GET {path}");
        assert_contains!(response.text, "<div id=\"app\">");
    }
    assert_eq!(app.get("/api/nope").await.status, 404, "an unknown api path must not quietly answer with html");
}

#[tokio::test]
async fn nothing_answers_on_the_environment_endpoints_any_more() {
    let app = app().await;
    assert_eq!(app.get("/api/environments").await.status, 404);
    assert_eq!(
        app.count("SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND UPPER(name) = 'ENVIRONMENT'", vec![]).await,
        0,
        "the table is dropped, not merely unused"
    );
}

// --- Local access

#[tokio::test]
async fn a_page_on_another_site_or_a_request_naming_another_host_is_refused_before_it_reaches_the_api() {
    let app = app().await;

    let from_another_site = app
        .client
        .post(app.url("/api/companies"))
        .header("Origin", "https://evil.example")
        .header("Content-Type", "application/json")
        .body("{\"name\":\"Evil\"}")
        .send()
        .await
        .unwrap();
    assert_eq!(from_another_site.status(), 403);
    assert!(app.get("/api/companies").await.list().is_empty());

    let port = app.running.as_ref().unwrap().port;
    let from_the_console =
        app.client.get(app.url("/api/companies")).header("Origin", format!("http://127.0.0.1:{port}")).send().await.unwrap();
    assert_eq!(from_the_console.status(), 200);

    let rebound = app.client.get(app.url("/api/companies")).header("Host", "rebind.attacker.test").send().await.unwrap();
    assert_eq!(rebound.status(), 403, "a Host that is not this machine is DNS rebinding");
}

// --- Diagrams

#[tokio::test]
async fn a_diagram_goes_in_whole_is_listed_read_back_shows_its_code_and_is_deleted_by_the_console() {
    let app = app().await;
    let folder = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(folder.path().join("src")).unwrap();
    std::fs::write(folder.path().join("src/order.rs"), "fn a() {}\nfn process_order() {\n    validate();\n}\n").unwrap();
    let company = app.a_company("Acme").await;
    let project = app.a_project_in(&company, "shop", folder.path()).await;
    let sheets = json!([{
        "title": "Orders",
        "mermaid": "flowchart LR\n  process[process_order] --> validate[Validate]",
        "links": {
            "process": [{"file": "src/order.rs", "startLine": 2, "endLine": 4}],
            "validate": [{"file": "src/order.rs", "startLine": 3}]
        }
    }]);

    let created = app.post("/api/diagrams", json!({ "projectId": project, "title": "Orders", "question": "How?", "sheets": sheets })).await;
    assert_eq!(created.status, 201, "{}", created.text);
    let diagram = created.str("id");
    assert_eq!(created.body.pointer("/sheets/0/kind"), Some(&json!("flowchart")));
    assert_eq!(created.body.pointer("/kinds"), Some(&json!(["flowchart"])));

    let listed = app.get("/api/diagrams").await;
    assert_eq!(listed.list().len(), 1);
    assert!(listed.list()[0].get("sheets").is_none(), "a list carries no sheets");

    let excerpt = app.get(&format!("/api/diagrams/{diagram}/source?file=src/order.rs&start=3")).await;
    assert_eq!(excerpt.status, 200, "{}", excerpt.text);
    assert_eq!(excerpt.get("highlightStart"), &json!(3));
    assert_eq!(excerpt.get("language"), &json!("rust"));
    let escaping = app.get(&format!("/api/diagrams/{diagram}/source?file=../../etc/hosts")).await;
    assert!(escaping.status == 400 || escaping.status == 404, "{}", escaping.text);

    let refused = app.post("/api/diagrams", json!({ "projectId": project, "title": "Bad", "sheets": [] })).await;
    assert_eq!(refused.status, 400);
    assert!(refused.detail().contains("sheets: a diagram needs at least one sheet"), "{}", refused.detail());

    assert_eq!(app.delete(&format!("/api/diagrams/{diagram}")).await.status, 204);
    assert_eq!(app.get(&format!("/api/diagrams/{diagram}")).await.status, 404);
}

#[tokio::test]
async fn a_session_stores_a_generated_diagram_through_mcp_only_when_its_spans_are_in_the_code() {
    let app = app().await;
    let folder = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(folder.path().join("src")).unwrap();
    std::fs::write(folder.path().join("src/order.rs"), "fn a() {}\nfn process_order() {\n    validate();\n}\n").unwrap();
    let company = app.a_company("Acme").await;
    let project = app.a_project_in(&company, "shop", folder.path()).await;
    let task = app.a_task(&project, "orders").await;
    let sheets = |end_line: u32| {
        json!([
            {
                "mermaid": "sequenceDiagram\n  participant O as Orders\n  participant V as Validator\n  O->>+V: validate()\n  V-->>-O: ok",
                "links": {
                    "O": [{"file": "src/order.rs", "startLine": 2, "endLine": end_line, "symbol": "process_order"}],
                    "#1": [{"file": "src/order.rs", "startLine": 3}]
                }
            },
            { "mermaid": "stateDiagram-v2\n  [*] --> Checked" }
        ])
    };
    let arguments = |anchors: &str, sheets: serde_json::Value| {
        json!({ "anchors": anchors, "title": "Orders", "question": "How is an order processed?", "sheets": sheets })
    };

    let refused = app.call_tool("rekall_diagram", arguments("project:shop task:orders", sheets(90))).await;
    assert!(refused.contains(r#"sheets[0].links["O"][0].endLine: line 90 is past the end of src/order.rs (4 lines)"#), "{refused}");
    assert!(app.get("/api/diagrams").await.list().is_empty());

    let answer = app.call_tool("rekall_diagram", arguments("project:shop task:orders", sheets(4))).await;
    assert!(answer.contains("2 sheets (sequence, state), 2 elements linked to code"), "{answer}");
    let stored = &app.get("/api/diagrams").await.list()[0];
    assert_eq!((stored["projectId"].as_str(), stored["taskId"].as_str()), (Some(project.as_str()), Some(task.as_str())));
    assert_eq!(stored["question"], json!("How is an order processed?"));

    let diagram = stored["id"].as_str().unwrap().to_string();
    let mut revised = arguments("project:shop task:orders", sheets(4).to_string().into());
    revised["diagram"] = json!(diagram);
    revised["title"] = json!("Orders, revised");
    assert!(app.call_tool("rekall_diagram", revised).await.contains("replaced"));
    let listed = app.get("/api/diagrams").await.list();
    assert_eq!((listed.len(), listed[0]["title"].as_str()), (1, Some("Orders, revised")));

    let on_project = app.call_tool("rekall_diagram", arguments("project:shop", sheets(4))).await;
    assert!(on_project.contains("written on `project:shop`"), "{on_project}");
    let newest = &app.get("/api/diagrams").await.list()[0];
    assert_eq!(newest["taskId"], json!(null));
}

#[tokio::test]
async fn a_session_stores_a_walkthrough_through_mcp_which_the_console_reads_back_and_the_gallery_marks() {
    let app = app().await;
    let folder = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(folder.path().join("src")).unwrap();
    std::fs::write(folder.path().join("src/order.rs"), "fn a() {}\nfn place() {\n    insert();\n}\n").unwrap();
    let company = app.a_company("Acme").await;
    app.a_project_in(&company, "shop", folder.path()).await;
    let entry = "sequenceDiagram\n  participant C as OrderController\n  participant S as OrderService\n  participant DB as Postgres\n  C->>+S: place(order)\n  S->>+DB: insert(order)\n  DB-->>-S: id\n  S-->>-C: Order";
    let walkthrough = |opens: usize| {
        json!({
            "anchors": "project:shop",
            "title": "Place an order",
            "question": "What happens when an order is placed?",
            "sheets": [
                {
                    "mermaid": entry,
                    "links": { "#1": [{"file": "src/order.rs", "startLine": 2, "endLine": 4, "symbol": "place"}] },
                    "walkthrough": {
                        "external": ["DB"],
                        "messages": { "#1": { "explain": "The service stores the order.", "input": "Order", "output": "Order with its id", "opens": opens } }
                    }
                },
                { "title": "Inside place", "mermaid": "sequenceDiagram\n  S->>S: validate(order)" }
            ]
        })
    };

    let refused = app.call_tool("rekall_diagram", walkthrough(5)).await;
    assert!(refused.contains(r##"sheets[0].walkthrough.messages["#1"].opens: there is no sheet 5"##), "{refused}");
    assert!(app.get("/api/diagrams").await.list().is_empty());

    let answer = app.call_tool("rekall_diagram", walkthrough(1)).await;
    assert!(answer.contains("2 sheets (sequence, sequence)"), "{answer}");
    let stored = &app.get("/api/diagrams").await.list()[0];
    assert_eq!(stored["walkthrough"], json!(true));
    let read = app.get(&format!("/api/diagrams/{}", stored["id"].as_str().unwrap())).await.body;
    assert_eq!(read["sheets"][0]["walkthrough"]["external"], json!(["DB"]));
    assert_eq!(read["sheets"][0]["walkthrough"]["messages"]["#1"]["opens"], json!(1));
    assert_eq!(read["sheets"][1]["walkthrough"], json!(null));
}

struct FixedFeed(Release);

#[async_trait]
impl ReleaseFeed for FixedFeed {
    async fn latest(&self) -> Result<Option<Release>, ReleaseFeedError> {
        Ok(Some(self.0.clone()))
    }
}

#[tokio::test]
async fn the_version_endpoint_reports_this_build_and_the_newer_release_the_feed_names() {
    let newer = Release { tag: "v999.0.0".into(), page_url: "https://example.test/release".into(), assets: vec![] };
    let options = StartOptions { no_restart: true, release_feed: Some(Arc::new(FixedFeed(newer))), ..StartOptions::default() };
    let app = app_with(&[], options).await;

    let version = app.get("/api/version").await;

    assert_eq!(version.status, 200);
    assert_eq!(version.str("current"), rekall_core::version::RUNNING_VERSION);
    assert_eq!(version.str("check"), "UPDATE_AVAILABLE");
    assert_eq!(version.get("latest")["version"], "999.0.0");
    assert_eq!(version.get("latest")["releaseUrl"], "https://example.test/release");
}

#[tokio::test]
async fn with_the_check_switched_off_the_version_endpoint_still_names_this_build() {
    let app = app_with(&[("rekall.update-check.enabled", "false")], StartOptions { no_restart: true, ..StartOptions::default() }).await;

    let version = app.get("/api/version").await;

    assert_eq!(version.str("check"), "DISABLED");
    assert!(version.get("latest").is_null());
}

#[tokio::test]
async fn a_sessions_transcript_in_a_projects_folder_reaches_the_token_tallies_of_its_task_and_step() {
    let app = app_with(&[("rekall.claude.transcript-scan-seconds", "1")], StartOptions { no_restart: true, ..StartOptions::default() }).await;
    let repo = app.folder.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    let company = app.a_company("Acme").await;
    let project = app.a_project_in(&company, "vega", &repo).await;
    let task = app.a_task(&project, "report").await;
    let step = app.a_step(&task, "Draft", None).await;
    assert_eq!(app.get("/api/token-usage").await.list(), Vec::<serde_json::Value>::new());

    let encoded: String = repo.to_string_lossy().chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '-' }).collect();
    let transcripts = app.folder.path().join("home/.claude/projects").join(encoded);
    std::fs::create_dir_all(&transcripts).unwrap();
    let call = |id: &str, output: u32| {
        format!(r#"{{"type":"assistant","timestamp":"2026-10-01T10:00:0{output}Z","message":{{"id":"{id}","content":[],"usage":{{"input_tokens":1,"output_tokens":{output},"cache_creation_input_tokens":0,"cache_read_input_tokens":0}}}}}}"#)
    };
    let lines = [
        r#"{"type":"user","timestamp":"2026-10-01T10:00:00Z","message":{"content":"/rk project:vega task:report step:1 start"}}"#.to_string(),
        call("m1", 3),
        r#"{"type":"user","timestamp":"2026-10-01T10:00:04Z","message":{"content":"/rk project:other task:elsewhere"}}"#.to_string(),
        call("m2", 5),
    ];
    std::fs::write(transcripts.join("s1.jsonl"), lines.join("\n")).unwrap();

    let mut tallies = Vec::new();
    for _ in 0..50 {
        tallies = app.get("/api/token-usage").await.list();
        if !tallies.is_empty() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    assert_eq!(tallies.len(), 1, "{tallies:?}");
    assert_eq!(tallies[0]["taskId"], json!(task));
    assert_eq!(tallies[0]["stepId"], json!(step));
    assert_eq!(tallies[0]["outputTokens"], json!(3));
    let days = app.get("/api/token-usage/daily").await.list();
    assert_eq!(days.len(), 1, "{days:?}");
    assert_eq!(days[0]["taskId"], json!(task));
    assert_eq!(days[0]["day"], json!("2026-10-01T00:00:00Z"));
    assert_eq!(days[0]["outputTokens"], json!(3));
    app.stop().await;
}
