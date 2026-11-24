//! A project's sections: written, edited, arranged in contexts and deleted from the console,
//! written by a session through `rekall_project_section`, and kept out of every task context.

mod support;

use serde_json::json;
use support::{app, id};

const ANCHORS: &str = "task:VE-1";

async fn titles(app: &support::App, project_id: &str) -> Vec<String> {
    app.get(&format!("/api/projects/{project_id}/sections")).await.list().iter().map(|s| s["title"].as_str().unwrap().to_string()).collect()
}

async fn contexts(app: &support::App, project_id: &str) -> Vec<(String, String)> {
    app.get(&format!("/api/projects/{project_id}/section-contexts"))
        .await
        .list()
        .iter()
        .map(|c| (c["id"].as_str().unwrap().to_string(), c["title"].as_str().unwrap().to_string()))
        .collect()
}

async fn context_titles(app: &support::App, project_id: &str) -> Vec<String> {
    contexts(app, project_id).await.into_iter().map(|(_, title)| title).collect()
}

async fn context_id(app: &support::App, project_id: &str, title: &str) -> String {
    contexts(app, project_id).await.into_iter().find(|(_, known)| known == title).unwrap().0
}

#[tokio::test]
async fn the_console_writes_edits_arranges_and_deletes_a_projects_sections() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;

    let deploy = id(&app.post(&format!("/api/projects/{project_id}/sections"), json!({ "title": "Deployment", "bodyMarkdown": "Blue-green." })).await);
    let glossary = id(&app.post(&format!("/api/projects/{project_id}/sections"), json!({ "title": "Glossary" })).await);
    assert_eq!(titles(&app, &project_id).await, ["Deployment", "Glossary"], "a new section goes last, and may start empty");
    assert_eq!(context_titles(&app, &project_id).await, ["General"], "a project with no context gets General for its first page");

    let renamed = app.put(&format!("/api/project-sections/{deploy}"), json!({ "title": "Release", "bodyMarkdown": "Rolling." })).await;
    assert_eq!((renamed.status, renamed.str("title"), renamed.str("bodyMarkdown")), (200, "Release".into(), "Rolling.".into()));
    let body_only = app.put(&format!("/api/project-sections/{deploy}"), json!({ "bodyMarkdown": "Canary." })).await;
    assert_eq!((body_only.str("title"), body_only.str("bodyMarkdown")), ("Release".into(), "Canary.".into()), "a field left out is kept");

    let general = context_id(&app, &project_id, "General").await;
    let team = id(&app.post(&format!("/api/projects/{project_id}/section-contexts"), json!({ "title": "Team" })).await);
    let arranged = app
        .put(
            &format!("/api/projects/{project_id}/sections/layout"),
            json!({ "contexts": [{ "contextId": team, "sectionIds": [glossary] }, { "contextId": general, "sectionIds": [deploy] }] }),
        )
        .await;
    assert_eq!(arranged.status, 200, "{}", arranged.text);
    assert_eq!(titles(&app, &project_id).await, ["Glossary", "Release"], "the contexts come in their order, and the pages inside each");
    assert_eq!(context_titles(&app, &project_id).await, ["Team", "General"]);
    let held_by: Vec<String> = app.get(&format!("/api/projects/{project_id}/sections")).await.list().iter().map(|s| s["contextId"].as_str().unwrap().to_string()).collect();
    assert_eq!(held_by, [team.clone(), general.clone()], "a page moves to another context by being arranged into it");

    app.delete(&format!("/api/project-sections/{glossary}")).await;
    assert_eq!(titles(&app, &project_id).await, ["Release"]);
    assert_eq!(app.delete(&format!("/api/project-sections/{glossary}")).await.status, 404);
}

#[tokio::test]
async fn a_title_is_unique_on_a_project_ignoring_case_and_an_arrangement_names_every_context_and_section_once() {
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

    let general = context_id(&app, &project_id, "General").await;
    let stranger = id(&app.post(&format!("/api/projects/{other_id}/section-contexts"), json!({ "title": "Other" })).await);
    for contexts in [
        json!([{ "contextId": general, "sectionIds": [first] }]),
        json!([{ "contextId": general, "sectionIds": [first, first] }]),
        json!([{ "contextId": general, "sectionIds": [first, second, first] }]),
        json!([{ "contextId": general, "sectionIds": [first, second] }, { "contextId": general, "sectionIds": [] }]),
        json!([{ "contextId": stranger, "sectionIds": [first, second] }]),
        json!([]),
    ] {
        let refused = app.put(&format!("/api/projects/{project_id}/sections/layout"), json!({ "contexts": contexts })).await;
        assert_eq!(refused.status, 400, "{contexts}: {}", refused.text);
    }
    assert_eq!(app.put(&format!("/api/projects/{project_id}/sections/layout"), json!({})).await.status, 400);
    assert_eq!(titles(&app, &project_id).await, ["deployment", "Glossary"]);
}

#[tokio::test]
async fn a_context_is_added_renamed_and_deleted_only_once_it_holds_no_page() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let other_id = app.a_project(&acme, "orion", "ACTIVE").await;
    let operations = id(&app.post(&format!("/api/projects/{project_id}/section-contexts"), json!({ "title": "Operations" })).await);

    assert_eq!(app.post(&format!("/api/projects/{project_id}/section-contexts"), json!({ "title": "operations" })).await.status, 409);
    assert_eq!(app.post(&format!("/api/projects/{project_id}/section-contexts"), json!({ "title": " " })).await.status, 400);
    assert_eq!(app.post(&format!("/api/projects/{other_id}/section-contexts"), json!({ "title": "Operations" })).await.status, 201, "another project is free to use it");
    let renamed = app.put(&format!("/api/section-contexts/{operations}"), json!({ "title": "Run" })).await;
    assert_eq!((renamed.status, renamed.str("title")), (200, "Run".into()));

    let page = id(&app.post(&format!("/api/projects/{project_id}/sections"), json!({ "title": "Deployment", "contextId": operations })).await);
    let refused = app.delete(&format!("/api/section-contexts/{operations}")).await;
    assert_eq!(refused.status, 409, "{}", refused.text);
    assert_contains!(refused.text, "still holds 1 page");
    assert_eq!(app.post(&format!("/api/projects/{other_id}/sections"), json!({ "title": "Elsewhere", "contextId": operations })).await.status, 400, "a page cannot go in another project's context");

    app.delete(&format!("/api/project-sections/{page}")).await;
    assert_eq!(app.delete(&format!("/api/section-contexts/{operations}")).await.status, 204);
    assert_eq!(context_titles(&app, &project_id).await, Vec::<String>::new());
    assert_eq!(app.delete(&format!("/api/section-contexts/{operations}")).await.status, 404);
}

#[tokio::test]
async fn a_session_writes_a_section_on_the_tasks_project_and_the_context_names_it_without_its_body() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    app.a_task(&project_id, "report-builder").await;

    let answer = app.call_tool("rekall_project_section", json!({ "anchors": ANCHORS, "title": "Deployment", "body": "Blue-green, behind the load balancer." })).await;

    assert_contains!(answer, "Section \"Deployment\" written on `project:vega`, in the context \"General\"", "1 section,");
    let sections = app.get(&format!("/api/projects/{project_id}/sections")).await.list();
    assert_eq!((sections.len(), sections[0]["bodyMarkdown"].as_str()), (1, Some("Blue-green, behind the load balancer.")));
    for anchors in [ANCHORS, "project:vega"] {
        let context = app.call_tool("rekall_context", json!({ "anchors": anchors })).await;
        assert_lacks!(context, "Blue-green");
        assert_contains!(context, "- Deployment\n</sections>");
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
async fn a_session_names_the_context_a_new_section_is_kept_in_and_a_person_reads_it_back_by_context() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    app.a_task(&project_id, "report-builder").await;

    let answer = app.call_tool("rekall_project_section", json!({ "anchors": ANCHORS, "title": "Runbook", "body": "Roll back first.", "context": "Operations" })).await;
    app.call_tool("rekall_project_section", json!({ "anchors": ANCHORS, "title": "Glossary", "body": "Terms.", "context": "operations" })).await;
    app.call_tool("rekall_project_section", json!({ "anchors": ANCHORS, "title": "Runbook", "body": "Roll back, then page.", "context": "Team", "replace": true })).await;

    assert_contains!(answer, "in the context \"Operations\"");
    assert_eq!(context_titles(&app, &project_id).await, ["Operations"], "a name matches ignoring case, and a replace never moves or adds");
    let listed = app.call_tool("rekall_read_sections", json!({ "anchors": "project:vega" })).await;
    assert_contains!(listed, "Operations:\n- Runbook", "- Glossary");
    assert_contains!(app.call_tool("rekall_read_sections", json!({ "anchors": "project:vega", "title": "runbook" })).await, "in the context \"Operations\"", "Roll back, then page.");
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
async fn deleting_a_project_takes_its_sections_and_contexts_with_it() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    app.post(&format!("/api/projects/{project_id}/sections"), json!({ "title": "Deployment" })).await;

    app.delete(&format!("/api/projects/{project_id}")).await;

    assert_eq!(app.count("SELECT COUNT(*) FROM project_section", vec![]).await, 0);
    assert_eq!(app.count("SELECT COUNT(*) FROM section_context", vec![]).await, 0);
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

#[tokio::test]
async fn a_session_regroups_every_page_by_concept_and_the_emptied_contexts_go() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    app.a_task(&project_id, "report-builder").await;
    for title in ["Deployment", "Glossary", "Runbook"] {
        app.post(&format!("/api/projects/{project_id}/sections"), json!({ "title": title })).await;
    }
    assert_eq!(context_titles(&app, &project_id).await, ["General"]);

    let answer = app
        .call_tool(
            "rekall_reorganize_sections",
            json!({ "anchors": "project:vega", "groups": [
                { "context": "Operations", "pages": ["runbook", "Deployment"] },
                { "context": "Team", "pages": ["Glossary"] }
            ] }),
        )
        .await;

    assert_contains!(answer, "`project:vega` reorganized: 3 pages changed context.", "One emptied context was dropped.", "- Operations: 2 pages", "- Team: 1 page");
    assert_eq!(context_titles(&app, &project_id).await, ["Operations", "Team"]);
    assert_eq!(titles(&app, &project_id).await, ["Runbook", "Deployment", "Glossary"], "the contexts and the pages in each come in the order sent");
}

#[tokio::test]
async fn a_reorganization_that_leaves_a_page_out_or_names_one_the_project_lacks_moves_nothing() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    for title in ["Deployment", "Glossary"] {
        app.post(&format!("/api/projects/{project_id}/sections"), json!({ "title": title })).await;
    }

    let left_out = app.call_tool("rekall_reorganize_sections", json!({ "anchors": "project:vega", "groups": [{ "context": "Ops", "pages": ["Deployment"] }] })).await;
    let unknown = app.call_tool("rekall_reorganize_sections", json!({ "anchors": "project:vega", "groups": [{ "context": "Ops", "pages": ["Deployment", "Glossary", "Ghost"] }] })).await;
    let missing = app.call_tool("rekall_reorganize_sections", json!({ "anchors": "project:vega" })).await;

    assert_contains!(left_out, "these were not: Glossary", "Nothing was moved");
    assert_contains!(unknown, "no page titled 'Ghost'");
    assert_contains!(missing, "needs its `groups`");
    assert_eq!(context_titles(&app, &project_id).await, ["General"]);
}
