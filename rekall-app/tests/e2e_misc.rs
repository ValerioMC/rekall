//! `RekallEndToEndTest`, part five: the export, companies, revisions, proposed steps, notes a
//! session writes, the path picker's folder listing, search, context size, the stateless MCP era over real HTTP, the console's routes, and local access.

mod support;

use std::collections::BTreeMap;
use std::io::Read;

use serde_json::json;
use support::{app, id, App};

/// Entry name to contents; directory entries kept as their own empty entries.
fn unzip(archive: &[u8]) -> BTreeMap<String, String> {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(archive)).unwrap();
    let mut entries = BTreeMap::new();
    for index in 0..zip.len() {
        let mut entry = zip.by_index(index).unwrap();
        let mut text = String::new();
        entry.read_to_string(&mut text).unwrap();
        entries.insert(entry.name().to_string(), text);
    }
    entries
}

async fn export(app: &App) -> BTreeMap<String, String> {
    let bytes = app.client.get(app.url("/api/export")).send().await.unwrap().bytes().await.unwrap();
    unzip(&bytes)
}

#[tokio::test]
async fn the_export_is_a_zip_of_company_project_task_note_md_shared_notes_under_each_task() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let vega = id(&app
        .post("/api/projects", json!({ "label": "vega", "title": "Vega Platform", "status": "ACTIVE", "companyId": acme }))
        .await);
    let validator = app.a_task(&vega, "report-builder").await;
    let retry = app.a_task(&vega, "retry-policy").await;
    app.a_task(&vega, "empty-one").await;

    app.post("/api/documents", json!({ "title": "CONTEXT.md", "kind": "context", "taskIds": [validator], "bodyMarkdown": "# Contesto\n\nprimo" }))
        .await;
    app.post(
        "/api/documents",
        json!({ "title": "kmaster14.md", "kind": "notes", "taskIds": [validator, retry], "bodyMarkdown": "Accesso via bastion." }),
    )
    .await;
    app.call_tool("rekall_wrapup", json!({ "anchors": "project:vega task:report-builder", "body": "## Stato\n\nGira." })).await;
    let aggregate = app.a_step(&validator, "Aggregate the rows", None).await;
    app.a_step(&validator, "Write the tests", Some("Un caso per settimana vuota.")).await;
    app.patch(&format!("/api/steps/{aggregate}"), json!({ "done": true })).await;

    let entries = export(&app).await;

    for name in [
        "Acme/vega/report-builder/CONTEXT.md",
        "Acme/vega/report-builder/kmaster14.md",
        "Acme/vega/retry-policy/kmaster14.md",
        "Acme/vega/empty-one/",
        "MANIFEST.md",
        "Acme/vega/report-builder/WRAPUP.md",
        "Acme/vega/report-builder/STEPS.md",
    ] {
        assert!(entries.contains_key(name), "{name} in {:?}", entries.keys());
    }
    assert_contains!(entries["Acme/vega/report-builder/STEPS.md"], "- [x] Aggregate the rows", "- [ ] Write the tests", "Un caso per settimana vuota.");
    assert_eq!(entries["Acme/vega/report-builder/CONTEXT.md"], "# Contesto\n\nprimo");
    assert_eq!(entries["Acme/vega/retry-policy/kmaster14.md"], "Accesso via bastion.");
    assert_contains!(
        entries["MANIFEST.md"],
        "company:Acme",
        "Vega Platform",
        "project:vega task:report-builder",
        "Notes that appear more than once",
        "Acme/vega/retry-policy/kmaster14.md",
        "no notes",
        "`WRAPUP.md`: the state of the implementation, written by Claude",
        "`STEPS.md`: 1 of 2 steps done"
    );
}

#[tokio::test]
async fn a_label_full_of_separators_cannot_escape_its_folder() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project = app.a_project(&acme, "../../etc", "ACTIVE").await;
    let task = app.a_task(&project, "a/b/c").await;
    app.post("/api/documents", json!({ "title": "../secret", "kind": "notes", "taskIds": [task], "bodyMarkdown": "x" })).await;

    let entries = export(&app).await;

    assert!(entries.keys().all(|path| !path.contains("..") && !path.starts_with('/')), "{:?}", entries.keys());
    assert!(entries.keys().any(|path| path.ends_with("secret.md")));
}

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

/// The Java test ran the migration's `REGEXP_REPLACE` on H2 itself; SQLite has no regex, so the
/// migration does the same in Rust, and this runs that.
#[test]
fn the_migrations_normalising_expression_turns_a_legacy_name_into_a_label() {
    for (legacy, expected) in [("Vega", "vega"), ("Progetto Vega", "progetto-vega"), ("../../etc", "etc"), ("a/b/c", "a-b-c")] {
        assert_eq!(rekall_repository::migration::legacy_label(legacy), expected, "{legacy}");
    }
}

#[tokio::test]
async fn a_project_cannot_exist_without_a_company() {
    let app = app().await;
    let refused = app.post("/api/projects", json!({ "label": "orphan", "title": "Orphan", "status": "ACTIVE" })).await;
    assert_eq!(refused.status, 404);
}

// --- Revisions

#[tokio::test]
async fn what_a_sessions_wrapup_replaces_and_what_a_delete_removes_stays_in_the_history_and_comes_back_on_restore() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    app.put(&format!("/api/tasks/{task_id}/wrapup"), json!({ "bodyMarkdown": "Written by hand." })).await;

    app.call_tool("rekall_wrapup", json!({ "anchors": "project:vega task:report-builder", "body": "Written by Claude." })).await;
    app.delete(&format!("/api/tasks/{task_id}/wrapup")).await;

    let revisions = app.get(&format!("/api/tasks/{task_id}/revisions?kind=WRAPUP")).await.list();
    let bodies: Vec<&str> = revisions.iter().map(|r| r["bodyMarkdown"].as_str().unwrap()).collect();
    assert_eq!(bodies, ["Written by Claude.", "Written by hand."]);

    let hand_written = revisions[1]["id"].as_str().unwrap();
    let restored = app.post(&format!("/api/tasks/{task_id}/revisions/{hand_written}/restore"), json!({})).await;

    assert_eq!(restored.str("kind"), "WRAPUP");
    assert_contains!(app.call_tool("rekall_context", json!({ "anchors": "project:vega task:report-builder" })).await, "Written by hand.");
}

#[tokio::test]
async fn a_description_edited_as_it_is_typed_keeps_the_version_from_before_the_editing_not_every_keystroke() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    for text in ["First brief.", "First brief, ed", "First brief, edited."] {
        app.put(
            &format!("/api/tasks/{task_id}"),
            json!({ "label": "report-builder", "title": "report-builder", "status": "TODO", "description": text, "projectId": project_id }),
        )
        .await;
    }

    let revisions = app.get(&format!("/api/tasks/{task_id}/revisions?kind=DESCRIPTION")).await.list();
    let bodies: Vec<&str> = revisions.iter().map(|r| r["bodyMarkdown"].as_str().unwrap()).collect();
    assert_eq!(bodies, ["First brief."]);
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

    // A session only adds, so it cannot overwrite a note by reusing its title.
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
    let random = rekall_common::Id::random();
    for path in [
        "/".to_string(),
        "/projects".into(),
        format!("/projects/{random}"),
        "/tasks".into(),
        "/report".into(),
        format!("/tasks/{random}"),
        "/search".into(),
        "/calendar".into(),
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
