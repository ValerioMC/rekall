//! A project's components: written, edited, ordered and deleted from the console, and handed to a
//! session in the context of the project and of each of its tasks.

mod support;

use serde_json::json;
use support::{app, id};

const ANCHORS: &str = "task:VE-1";

async fn names(app: &support::App, project_id: &str) -> Vec<String> {
    app.get(&format!("/api/projects/{project_id}/components")).await.list().iter().map(|c| c["name"].as_str().unwrap().to_string()).collect()
}

#[tokio::test]
async fn the_console_adds_edits_orders_and_deletes_a_projects_components() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let url = format!("/api/projects/{project_id}/components");

    let api = id(&app.post(&url, json!({ "name": "API", "folder": "/work/vega/api", "description": "The REST surface." })).await);
    let ui = id(&app.post(&url, json!({ "name": "Console", "folder": "/work/vega/ui" })).await);
    assert_eq!(names(&app, &project_id).await, ["API", "Console"], "a new component goes last, and may start without a description");

    let renamed = app.put(&format!("/api/project-components/{api}"), json!({ "name": "Gateway", "description": "Routes." })).await;
    assert_eq!((renamed.status, renamed.str("name"), renamed.str("folder"), renamed.str("description")), (200, "Gateway".into(), "/work/vega/api".into(), "Routes.".into()), "a field left out is kept");

    let order = app.put(&format!("{url}/order"), json!({ "componentIds": [ui, api] })).await;
    assert_eq!(order.status, 200);
    assert_eq!(names(&app, &project_id).await, ["Console", "Gateway"]);

    app.delete(&format!("/api/project-components/{ui}")).await;
    assert_eq!(names(&app, &project_id).await, ["Gateway"]);
    assert_eq!(app.delete(&format!("/api/project-components/{ui}")).await.status, 404);
}

#[tokio::test]
async fn a_component_without_a_name_takes_its_folders_and_a_trailing_slash_is_dropped() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;

    let created = app.post(&format!("/api/projects/{project_id}/components"), json!({ "folder": "/work/vega/billing-service/" })).await;

    assert_eq!((created.status, created.str("name"), created.str("folder")), (201, "billing-service".into(), "/work/vega/billing-service".into()));
}

#[tokio::test]
async fn a_name_is_unique_on_a_project_ignoring_case_and_an_order_names_every_component_once() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let other_id = app.a_project(&acme, "orion", "ACTIVE").await;
    let url = format!("/api/projects/{project_id}/components");
    let first = id(&app.post(&url, json!({ "name": "API", "folder": "/a" })).await);
    let second = id(&app.post(&url, json!({ "name": "Console", "folder": "/b" })).await);

    assert_eq!(app.post(&url, json!({ "name": "api", "folder": "/c" })).await.status, 409);
    assert_eq!(app.put(&format!("/api/project-components/{second}"), json!({ "name": "API" })).await.status, 409);
    assert_eq!(app.put(&format!("/api/project-components/{first}"), json!({ "name": "api" })).await.status, 200, "its own name, recased");
    assert_eq!(app.post(&format!("/api/projects/{other_id}/components"), json!({ "name": "API", "folder": "/a" })).await.status, 201, "another project is free to use it");
    assert_eq!(app.post(&url, json!({ "name": "Docs", "folder": "  " })).await.status, 400);
    assert_eq!(app.put(&format!("/api/project-components/{first}"), json!({ "name": "  " })).await.status, 400);
    assert_eq!(app.put(&format!("/api/project-components/{first}"), json!({ "folder": " " })).await.status, 400);

    for ids in [json!([first]), json!([first, first]), json!([first, second, first])] {
        let refused = app.put(&format!("{url}/order"), json!({ "componentIds": ids })).await;
        assert_eq!(refused.status, 400, "{ids}: {}", refused.text);
    }
    assert_eq!(names(&app, &project_id).await, ["api", "Console"]);
}

#[tokio::test]
async fn the_context_of_the_project_and_of_its_tasks_lists_the_components_and_the_task_does_not_repeat_them() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    app.a_task(&project_id, "report-builder").await;
    let url = format!("/api/projects/{project_id}/components");
    app.post(&url, json!({ "name": "API", "folder": "/work/vega/api", "summary": "The REST surface.", "description": "## Purpose\nSecret detail." })).await;
    app.post(&url, json!({ "name": "Console", "folder": "/work/vega/ui" })).await;

    for anchors in [ANCHORS, "project:vega"] {
        let context = app.call_tool("rekall_context", json!({ "anchors": anchors })).await;
        assert_contains!(
            context,
            "<components count=\"2\">",
            "<component name=\"API\" folder=\"/work/vega/api\">\nThe REST surface.\n</component>",
            "<component name=\"Console\" folder=\"/work/vega/ui\">\n</component>"
        );
        assert_eq!(context.matches("<components").count(), 1, "{anchors}: once, under the project");
        assert_lacks!(context, "Secret detail");
    }
    let task_only = app.call_tool("rekall_context", json!({ "anchors": "task:VE-1" })).await;
    assert_contains!(task_only, "<component name=\"API\"");
}

#[tokio::test]
async fn a_project_without_components_renders_none() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    app.a_task(&project_id, "report-builder").await;

    assert_lacks!(app.call_tool("rekall_context", json!({ "anchors": ANCHORS })).await, "<components");
}

#[tokio::test]
async fn deleting_a_project_takes_its_components_with_it() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    app.post(&format!("/api/projects/{project_id}/components"), json!({ "name": "API", "folder": "/a" })).await;

    app.delete(&format!("/api/projects/{project_id}")).await;

    assert_eq!(app.count("SELECT COUNT(*) FROM project_component", vec![]).await, 0);
}

const PAGE: &str = "## Purpose\nIssues invoices.\n\n## Interfaces\n| Route | Does |\n|---|---|\n| POST /invoices | Creates one |";

#[tokio::test]
async fn a_session_writes_a_component_and_rewrites_it_by_name_or_by_folder_keeping_the_name() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let written = json!({ "anchors": "project:vega", "name": "Billing", "folder": "/work/vega/billing", "summary": "Charges customers.", "description": PAGE });

    let first = app.call_tool("rekall_component", written.clone()).await;
    assert_contains!(first, "Component \"Billing\" written on `project:vega`", "1 component");

    let by_name = app.call_tool("rekall_component", json!({ "anchors": "project:vega", "name": "billing", "folder": "/work/vega/billing", "summary": "Invoices and charges.", "description": PAGE })).await;
    assert_contains!(by_name, "rewritten");
    let by_folder = app.call_tool("rekall_component", json!({ "anchors": "project:vega", "name": "Invoicing", "folder": "/work/vega/billing/", "summary": "Invoices and charges, again.", "description": PAGE })).await;
    assert_contains!(by_folder, "Component \"Billing\" on `project:vega` rewritten");

    let rows = app.get(&format!("/api/projects/{project_id}/components")).await.list();
    assert_eq!(rows.len(), 1);
    assert_eq!((rows[0]["name"].as_str(), rows[0]["summary"].as_str()), (Some("Billing"), Some("Invoices and charges, again.")));
}

#[tokio::test]
async fn a_session_reads_the_components_listed_by_summary_and_one_whole() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    app.a_project(&acme, "vega", "ACTIVE").await;
    app.call_tool("rekall_component", json!({ "anchors": "project:vega", "name": "Billing", "folder": "/work/vega/billing", "summary": "Charges customers.", "description": PAGE })).await;

    let listed = app.call_tool("rekall_read_components", json!({ "anchors": "project:vega" })).await;
    assert_contains!(listed, "1 component", "- Billing (`/work/vega/billing`", "Charges customers.");
    assert_lacks!(listed, "POST /invoices");

    let whole = app.call_tool("rekall_read_components", json!({ "anchors": "project:vega", "name": "billing" })).await;
    assert_contains!(whole, "Component \"Billing\"", "POST /invoices");
    assert_contains!(app.call_tool("rekall_read_components", json!({ "anchors": "project:vega", "name": "Ledger" })).await, "no component named 'Ledger'", "has: Billing");
}

#[tokio::test]
async fn a_summary_that_is_missing_or_too_long_is_refused_and_nothing_is_written() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let long = "s".repeat(501);

    let too_long = app.call_tool("rekall_component", json!({ "anchors": "project:vega", "name": "Billing", "folder": "/b", "summary": long, "description": PAGE })).await;
    let missing = app.call_tool("rekall_component", json!({ "anchors": "project:vega", "name": "Billing", "folder": "/b", "summary": "  ", "description": PAGE })).await;

    assert_contains!(too_long, "capped at 500");
    assert_contains!(missing, "'summary' is required");
    assert!(names(&app, &project_id).await.is_empty());
}

#[tokio::test]
async fn a_task_context_holds_a_components_description_only_when_the_task_names_it() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    app.a_task(&project_id, "report-builder").await;
    app.call_tool("rekall_component", json!({ "anchors": "project:vega", "name": "Billing", "folder": "/work/vega/billing", "summary": "Charges customers.", "description": PAGE })).await;

    assert_lacks!(app.call_tool("rekall_context", json!({ "anchors": ANCHORS })).await, "POST /invoices");

    app.call_tool("rekall_description", json!({ "anchors": ANCHORS, "body": "Change how [[Billing]] rounds." })).await;
    let named = app.call_tool("rekall_context", json!({ "anchors": ANCHORS })).await;
    assert_contains!(named, "**Billing** (component, folder `/work/vega/billing`)", "POST /invoices");
}
