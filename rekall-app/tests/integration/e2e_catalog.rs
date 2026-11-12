//! `RekallEndToEndTest`, part one: the catalog entered through the console's API and read back
//! over MCP — anchors, labels, notes on many tasks, cascades.

mod support;

use serde_json::json;
use support::{app, id};

#[tokio::test]
async fn one_anchored_call_brings_back_the_task_its_project_and_every_note_on_it() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = id(&app
        .post(
            "/api/projects",
            json!({ "label": "vega", "title": "Vega Platform", "status": "ACTIVE", "description": "Progetto Vega", "companyId": acme }),
        )
        .await);
    app.a_project(&acme, "beacon", "PAUSED").await;
    let task_id = id(&app
        .post(
            "/api/tasks",
            json!({ "label": "report-builder-main-workflow", "title": "Report builder, main workflow", "status": "IN_PROGRESS", "projectId": project_id }),
        )
        .await);

    app.post(
        "/api/documents",
        json!({ "title": "CONTEXT.md", "kind": "context", "taskId": task_id, "bodyMarkdown": "# Contesto\n\nIl workflow parte da POST /api/v1/pipelines." }),
    )
    .await;
    app.post(
        "/api/documents",
        json!({ "title": "kmaster14.md", "kind": "notes", "taskId": task_id, "bodyMarkdown": "Cluster kmaster14, accesso via bastion." }),
    )
    .await;

    let context = app.call_tool("rekall_context", json!({ "anchors": "project:vega task:report-builder-main-workflow" })).await;

    assert_contains!(
        context,
        "Project: Vega Platform",
        "Task: Report builder, main workflow",
        "`project:vega`",
        "`task:report-builder-main-workflow`",
        "POST /api/v1/pipelines",
        "Cluster kmaster14, accesso via bastion."
    );
    assert_lacks!(context, "beacon");
    assert_eq!(context.split("Project: Vega Platform").count(), 2, "no record is rendered twice");
}

#[tokio::test]
async fn a_tasks_description_arrives_as_a_document_not_as_a_bullet() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let description = "## Cosa deve fare\n\nIl report builder genera il report settimanale.\n\n## Fuori scope\n\nIl confronto fra settimane diverse.";
    app.post(
        "/api/tasks",
        json!({ "label": "report-builder", "title": "Report builder", "status": "IN_PROGRESS", "description": description, "projectId": project_id }),
    )
    .await;

    let context = app.call_tool("rekall_context", json!({ "anchors": "task:report-builder" })).await;

    assert_contains!(context, "<description>", "</description>", "## Fuori scope", "Il confronto fra settimane diverse.");
    assert_lacks!(context, "- `description`");
}

#[tokio::test]
async fn the_title_can_be_rewritten_and_the_anchor_still_loads_the_record() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = id(&app
        .post("/api/tasks", json!({ "label": "report-builder", "title": "Validator", "status": "TODO", "projectId": project_id }))
        .await);

    app.put(
        &format!("/api/tasks/{task_id}"),
        json!({ "label": "report-builder", "title": "Report builder, main workflow", "status": "IN_PROGRESS", "projectId": project_id }),
    )
    .await;

    let context = app.call_tool("rekall_context", json!({ "anchors": "task:report-builder" })).await;
    assert_contains!(context, "Task: Report builder, main workflow", "IN_PROGRESS");
}

#[tokio::test]
async fn a_label_typed_as_a_sentence_is_stored_as_the_slug_the_anchor_can_carry() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project = app
        .post("/api/projects", json!({ "label": "  Vega Platform ", "title": "Vega Platform", "status": "ACTIVE", "companyId": acme }))
        .await;

    assert_eq!(project.str("label"), "vega-platform");
    assert_eq!(project.str("anchor"), "project:vega-platform");
    assert_contains!(app.call_tool("rekall_context", json!({ "anchors": "project:vega-platform" })).await, "Project: Vega Platform");
}

#[tokio::test]
async fn a_label_with_nothing_usable_left_in_it_is_refused_as_a_bad_request() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let refused = app
        .post("/api/projects", json!({ "label": "///", "title": "Nowhere", "status": "ACTIVE", "companyId": acme }))
        .await;
    assert_eq!(refused.status, 400);
}

#[tokio::test]
async fn two_companies_cannot_share_a_name_in_any_case_and_a_rename_to_its_own_name_is_fine() {
    let app = app().await;
    let acme = app.a_company("Acme").await;

    let second = app.post("/api/companies", json!({ "name": "ACME" })).await;
    let renamed = app.put(&format!("/api/companies/{acme}"), json!({ "name": "Acme", "description": "same name" })).await;

    assert_eq!(second.status, 409);
    assert_contains!(second.detail(), "already exists");
    assert_eq!(renamed.status, 200);
}

#[tokio::test]
async fn two_projects_of_one_company_cannot_share_a_label_but_two_companies_can() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let globex = app.a_company("Globex").await;
    app.a_project(&acme, "website", "ACTIVE").await;

    let clash = app.post("/api/projects", json!({ "label": "website", "title": "Again", "status": "ACTIVE", "companyId": acme })).await;
    let elsewhere = app.post("/api/projects", json!({ "label": "website", "title": "Theirs", "status": "ACTIVE", "companyId": globex })).await;

    assert_eq!(clash.status, 409);
    assert_contains!(clash.detail(), "already uses that label");
    assert_eq!(elsewhere.status, 201);
}

#[tokio::test]
async fn two_tasks_on_one_project_cannot_share_a_label_and_the_refusal_says_why() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    app.post("/api/tasks", json!({ "label": "setup", "title": "Setup", "status": "TODO", "projectId": project_id })).await;

    let second = app
        .post("/api/tasks", json!({ "label": "Setup", "title": "Setup again", "status": "TODO", "projectId": project_id }))
        .await;

    assert_eq!(second.status, 409);
    assert_contains!(second.detail(), "already uses that label");
}

#[tokio::test]
async fn changing_a_label_changes_the_anchor_and_the_old_one_stops_resolving() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;

    app.put(
        &format!("/api/projects/{project_id}"),
        json!({ "label": "vega-2", "title": "Vega", "status": "ACTIVE", "companyId": acme }),
    )
    .await;

    assert_contains!(app.call_tool("rekall_context", json!({ "anchors": "project:vega-2" })).await, "Project: Vega");
    assert_contains!(app.call_tool("rekall_context", json!({ "anchors": "project:vega" })).await, "No project matches 'vega'");
}

#[tokio::test]
async fn a_projects_folder_reaches_its_tasks_and_a_blank_one_clears_it() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    let update = |folder: &'static str| {
        let (app, project_id, acme) = (&app, project_id.clone(), acme.clone());
        async move {
            app.put(
                &format!("/api/projects/{project_id}"),
                json!({ "label": "vega", "title": "Vega", "status": "ACTIVE", "companyId": acme, "repoFolder": folder }),
            )
            .await
        }
    };

    let saved = update("/Users/someone/Projects/vega").await;
    assert_eq!(saved.str("repoFolder"), "/Users/someone/Projects/vega");
    assert_eq!(app.get_task(&task_id).await["projectRepoFolder"], "/Users/someone/Projects/vega");

    assert!(update("   ").await.get("repoFolder").is_null());
}

#[tokio::test]
async fn a_note_belongs_to_the_task_it_is_written_on() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let validator = app.a_task(&project_id, "report-builder").await;
    let retry = app.a_task(&project_id, "retry-policy").await;

    let note = app
        .post("/api/documents", json!({ "title": "kmaster14.md", "kind": "notes", "taskId": validator, "bodyMarkdown": "Accesso via bastion." }))
        .await;

    assert_eq!(note.str("taskId"), validator);
    assert_contains!(app.call_tool("rekall_context", json!({ "anchors": "task:report-builder" })).await, "Accesso via bastion.");
    assert_lacks!(app.call_tool("rekall_context", json!({ "anchors": "task:retry-policy" })).await, "Accesso via bastion.");
    assert!(app.documents_on(&retry).await.is_empty());
}

#[tokio::test]
async fn a_note_needs_a_task_and_cannot_move_to_another() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let first = app.a_task(&project_id, "a").await;
    let second = app.a_task(&project_id, "b").await;

    let without_task = app.post("/api/documents", json!({ "title": "x", "kind": "notes" })).await;
    let note = id(&app.post("/api/documents", json!({ "title": "x", "kind": "notes", "taskId": first })).await);
    let moved = app.put(&format!("/api/documents/{note}"), json!({ "title": "x", "kind": "notes", "taskId": second })).await;

    assert_eq!(without_task.status, 400);
    assert_eq!(moved.status, 400);
    assert_eq!(app.documents_on(&first).await.len(), 1, "still on the task it was written on");
    assert!(app.documents_on(&second).await.is_empty());
}

#[tokio::test]
async fn deleting_a_task_deletes_its_notes_and_only_its_own() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let doomed = app.a_task(&project_id, "doomed").await;
    let survivor = app.a_task(&project_id, "survivor").await;
    app.post("/api/documents", json!({ "title": "doomed.md", "kind": "notes", "taskId": doomed, "bodyMarkdown": "sola" })).await;
    app.post("/api/documents", json!({ "title": "survivor.md", "kind": "notes", "taskId": survivor, "bodyMarkdown": "resta" })).await;

    app.delete(&format!("/api/tasks/{doomed}")).await;

    assert_eq!(app.documents_on(&survivor).await.len(), 1);
    assert_eq!(app.count("SELECT COUNT(*) FROM document", vec![]).await, 1);
}

#[tokio::test]
async fn deleting_a_project_takes_its_tasks_and_the_notes_it_owns() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task = app.a_task(&project_id, "report-builder").await;
    app.post("/api/documents", json!({ "title": "n.md", "kind": "notes", "taskId": task, "bodyMarkdown": "x" })).await;

    app.delete(&format!("/api/projects/{project_id}")).await;

    assert_eq!(app.count("SELECT COUNT(*) FROM task", vec![]).await, 0);
    assert_eq!(app.count("SELECT COUNT(*) FROM document", vec![]).await, 0);
}

#[tokio::test]
async fn a_single_project_anchor_lists_its_tasks_as_anchors_without_their_bodies() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder-main-workflow").await;
    app.post("/api/documents", json!({ "title": "CONTEXT.md", "kind": "context", "taskId": task_id, "bodyMarkdown": "segreto" })).await;

    let context = app.call_tool("rekall_context", json!({ "anchors": "project:vega" })).await;

    assert_contains!(context, "`task:report-builder-main-workflow`");
    assert_lacks!(context, "segreto");
}

#[tokio::test]
async fn a_bare_term_resolves_when_it_is_unambiguous_and_reports_the_candidates_when_it_is_not() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    app.a_task(&project_id, "vega").await;

    assert_contains!(app.call_tool("rekall_context", json!({ "anchors": "report-builder" })).await, "Nothing matches 'report-builder'");
    assert_contains!(
        app.call_tool("rekall_context", json!({ "anchors": "vega" })).await,
        "matches 2 records",
        "Qualify it as `entity:value`"
    );
}

#[tokio::test]
async fn a_task_label_that_two_projects_share_is_disambiguated_by_the_project_anchor() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let vega = app.a_project(&acme, "vega", "ACTIVE").await;
    let beacon = app.a_project(&acme, "beacon", "ACTIVE").await;
    app.post("/api/tasks", json!({ "label": "setup", "title": "Setup", "status": "TODO", "projectId": vega })).await;
    app.post("/api/tasks", json!({ "label": "setup", "title": "Setup", "status": "DONE", "projectId": beacon })).await;

    assert_contains!(app.call_tool("rekall_context", json!({ "anchors": "task:setup" })).await, "matches 2 records");
    assert_contains!(
        app.call_tool("rekall_context", json!({ "anchors": "project:beacon task:setup" })).await,
        "Task: Setup",
        "DONE"
    );
}

#[tokio::test]
async fn a_project_can_be_archived_and_stays_listed_with_that_status() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    app.a_project(&acme, "attic", "ARCHIVED").await;

    let projects = app.get("/api/projects").await;

    assert_eq!(projects.body[0]["status"], "ARCHIVED");
}

// Asserted as an exact list so adding a read or write tool breaks a test.
#[tokio::test]
async fn the_mcp_endpoint_exposes_four_ways_to_read_and_ten_writes() {
    let app = app().await;
    let answer = app.rpc("tools/list", json!({})).await;
    let mut names: Vec<String> = answer["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool["name"].as_str().unwrap().to_string())
        .collect();
    names.sort();
    assert_eq!(
        names,
        [
            "rekall_assess_vulnerability",
            "rekall_context",
            "rekall_create_task",
            "rekall_delete_step",
            "rekall_description",
            "rekall_deviation",
            "rekall_diagram",
            "rekall_feedback",
            "rekall_inventory",
            "rekall_join_epic",
            "rekall_note",
            "rekall_project_section",
            "rekall_propose_flow",
            "rekall_propose_step",
            "rekall_read_sections",
            "rekall_record_commit",
            "rekall_replan",
            "rekall_search",
            "rekall_step",
            "rekall_vulnerabilities",
            "rekall_wrapup"
        ]
    );
}
