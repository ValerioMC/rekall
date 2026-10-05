//! A project's sections: written, edited, ordered and deleted from the console, written by a
//! session through `rekall_project_section`, and kept out of every context.

mod support;

use serde_json::json;
use support::{app, id};

const ANCHORS: &str = "project:vega task:report-builder";

async fn titles(app: &support::App, project_id: &str) -> Vec<String> {
    app.get(&format!("/api/projects/{project_id}/sections")).await.list().iter().map(|s| s["title"].as_str().unwrap().to_string()).collect()
}

#[tokio::test]
async fn the_console_writes_edits_orders_and_deletes_a_projects_sections() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;

    let deploy = id(&app.post(&format!("/api/projects/{project_id}/sections"), json!({ "title": "Deployment", "bodyMarkdown": "Blue-green." })).await);
    let glossary = id(&app.post(&format!("/api/projects/{project_id}/sections"), json!({ "title": "Glossary" })).await);
    assert_eq!(titles(&app, &project_id).await, ["Deployment", "Glossary"], "a new section goes last, and may start empty");

    let renamed = app.put(&format!("/api/project-sections/{deploy}"), json!({ "title": "Release", "bodyMarkdown": "Rolling." })).await;
    assert_eq!((renamed.status, renamed.str("title"), renamed.str("bodyMarkdown")), (200, "Release".into(), "Rolling.".into()));
    let body_only = app.put(&format!("/api/project-sections/{deploy}"), json!({ "bodyMarkdown": "Canary." })).await;
    assert_eq!((body_only.str("title"), body_only.str("bodyMarkdown")), ("Release".into(), "Canary.".into()), "a field left out is kept");

    let order = app.put(&format!("/api/projects/{project_id}/sections/order"), json!({ "sectionIds": [glossary, deploy] })).await;
    assert_eq!(order.status, 200);
    assert_eq!(titles(&app, &project_id).await, ["Glossary", "Release"]);

    app.delete(&format!("/api/project-sections/{glossary}")).await;
    assert_eq!(titles(&app, &project_id).await, ["Release"]);
    assert_eq!(app.delete(&format!("/api/project-sections/{glossary}")).await.status, 404);
}

#[tokio::test]
async fn a_title_is_unique_on_a_project_ignoring_case_and_an_order_names_every_section_once() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let other_id = app.a_project(&acme, "orion", "ACTIVE").await;
    let first = id(&app.post(&format!("/api/projects/{project_id}/sections"), json!({ "title": "Deployment" })).await);
    let second = id(&app.post(&format!("/api/projects/{project_id}/sections"), json!({ "title": "Glossary" })).await);

    assert_eq!(app.post(&format!("/api/projects/{project_id}/sections"), json!({ "title": "deployment" })).await.status, 409);
    assert_eq!(app.put(&format!("/api/project-sections/{second}"), json!({ "title": "DEPLOYMENT" })).await.status, 409);
    assert_eq!(app.put(&format!("/api/project-sections/{first}"), json!({ "title": "deployment" })).await.status, 200, "its own title, recased");
    assert_eq!(app.post(&format!("/api/projects/{other_id}/sections"), json!({ "title": "Deployment" })).await.status, 201, "another project is free to use it");
    assert_eq!(app.post(&format!("/api/projects/{project_id}/sections"), json!({ "title": "  " })).await.status, 400);

    for ids in [json!([first]), json!([first, first]), json!([first, second, first])] {
        let refused = app.put(&format!("/api/projects/{project_id}/sections/order"), json!({ "sectionIds": ids })).await;
        assert_eq!(refused.status, 400, "{ids}: {}", refused.text);
    }
    assert_eq!(titles(&app, &project_id).await, ["deployment", "Glossary"]);
}

#[tokio::test]
async fn a_session_writes_a_section_on_the_tasks_project_and_no_context_loads_it() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    app.a_task(&project_id, "report-builder").await;

    let answer = app.call_tool("rekall_project_section", json!({ "anchors": ANCHORS, "title": "Deployment", "body": "Blue-green, behind the load balancer." })).await;

    assert_contains!(answer, "Section \"Deployment\" written on `project:vega`", "1 section,");
    let sections = app.get(&format!("/api/projects/{project_id}/sections")).await.list();
    assert_eq!((sections.len(), sections[0]["bodyMarkdown"].as_str()), (1, Some("Blue-green, behind the load balancer.")));
    for anchors in [ANCHORS, "project:vega"] {
        let context = app.call_tool("rekall_context", json!({ "anchors": anchors })).await;
        assert_lacks!(context, "Blue-green", "Deployment");
    }

    assert_contains!(
        app.call_tool("rekall_project_section", json!({ "anchors": ANCHORS, "title": "deployment", "body": "Something else." })).await,
        "already has a section titled"
    );
    assert_contains!(
        app.call_tool("rekall_project_section", json!({ "anchors": "project:vega", "title": "x", "body": "y" })).await,
        "No task in those anchors"
    );
    assert_contains!(
        app.call_tool("rekall_project_section", json!({ "anchors": ANCHORS, "title": "x", "body": "  " })).await,
        "'body' is required"
    );
    assert_eq!(titles(&app, &project_id).await, ["Deployment"]);
}

#[tokio::test]
async fn a_session_rewrites_a_section_by_its_title_when_asked_to_replace_and_the_title_keeps_its_spelling() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    app.a_task(&project_id, "report-builder").await;
    let section = id(&app.post(&format!("/api/projects/{project_id}/sections"), json!({ "title": "Deployment", "bodyMarkdown": "Old." })).await);

    let answer = app.call_tool("rekall_project_section", json!({ "anchors": ANCHORS, "title": "deployment", "body": "New.", "replace": true })).await;

    assert_contains!(answer, "Section \"Deployment\" on `project:vega` rewritten");
    let sections = app.get(&format!("/api/projects/{project_id}/sections")).await.list();
    assert_eq!(sections.len(), 1, "rewritten in place, not added");
    assert_eq!((sections[0]["id"].as_str(), sections[0]["title"].as_str(), sections[0]["bodyMarkdown"].as_str()), (Some(section.as_str()), Some("Deployment"), Some("New.")));

    assert_contains!(
        app.call_tool("rekall_project_section", json!({ "anchors": ANCHORS, "title": "Glossary", "body": "Terms.", "replace": true })).await,
        "written on `project:vega`",
        "2 sections,"
    );
}

#[tokio::test]
async fn deleting_a_project_takes_its_sections_with_it() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    app.post(&format!("/api/projects/{project_id}/sections"), json!({ "title": "Deployment" })).await;

    app.delete(&format!("/api/projects/{project_id}")).await;

    assert_eq!(app.count("SELECT COUNT(*) FROM project_section", vec![]).await, 0);
}

#[tokio::test]
async fn a_change_to_a_projects_sections_reaches_every_open_console_as_a_project_section_frame() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let mut stream = app.open_stream().await;

    app.post(&format!("/api/projects/{project_id}/sections"), json!({ "title": "Deployment" })).await;

    let frame = stream.await_line("event:project-section", 5).await;
    assert_contains!(frame, "project-section");
    assert_contains!(stream.await_line(&project_id, 5).await, &project_id);
}
