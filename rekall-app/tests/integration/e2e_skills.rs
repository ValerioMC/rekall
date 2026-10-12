//! `RekallEndToEndTest`, skills: assigned to a task, a step or a tag from the console, and handed
//! to a session in the context it loads.

mod support;

use serde_json::json;
use support::{app, id};

#[tokio::test]
async fn a_task_step_and_tag_skills_all_reach_the_context_with_their_scope() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let tag = id(&app.post("/api/tags", json!({ "name": "Backend", "icon": "gear", "color": "teal" })).await);
    let task_id = app.a_task(&project_id, "report-builder").await;
    app.put(&format!("/api/tasks/{task_id}"), json!({ "label": "report-builder", "title": "report-builder", "status": "TODO", "projectId": project_id, "tagId": tag })).await;
    let step_id = app.a_step(&task_id, "Design the picker", Some("Un selettore.")).await;

    assert_eq!(app.put(&format!("/api/tasks/{task_id}/skills"), json!({ "skills": ["rust-standards"] })).await.status, 200);
    assert_eq!(app.put(&format!("/api/tags/{tag}/skills"), json!({ "skills": ["code-review", "rust-standards"] })).await.status, 200);
    assert_eq!(app.put(&format!("/api/steps/{step_id}/skills"), json!({ "skills": ["ui-ux-pro-max"] })).await.status, 200);

    let context = app.call_tool("rekall_context", json!({ "anchors": "task:report-builder" })).await;

    assert_contains!(
        context,
        "<skills count=\"3\">",
        "- `rust-standards` (task)",
        "- `code-review` (tag: Backend)",
        "- `ui-ux-pro-max` (step: Design the picker)"
    );
}

#[tokio::test]
async fn a_finished_steps_skill_is_no_longer_handed_over() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    let step_id = app.a_step(&task_id, "Design the picker", None).await;
    app.put(&format!("/api/steps/{step_id}/skills"), json!({ "skills": ["ui-ux-pro-max"] })).await;

    app.patch(&format!("/api/steps/{step_id}"), json!({ "done": true })).await;

    let context = app.call_tool("rekall_context", json!({ "anchors": "task:report-builder" })).await;
    assert_lacks!(context, "<skills", "ui-ux-pro-max");
}

#[tokio::test]
async fn a_write_replaces_the_list_and_the_listing_reports_every_owner() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    app.put(&format!("/api/tasks/{task_id}/skills"), json!({ "skills": ["one", "two"] })).await;

    app.put(&format!("/api/tasks/{task_id}/skills"), json!({ "skills": ["two", "three"] })).await;

    let listing = app.get("/api/skill-assignments").await;
    let names: Vec<String> = listing.get("taskSkills").as_array().unwrap().iter().map(|row| row["skillName"].as_str().unwrap().to_string()).collect();
    assert_eq!(names, ["two", "three"]);
}

#[tokio::test]
async fn a_malformed_name_is_a_400_and_an_unknown_owner_a_404() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;

    assert_eq!(app.put(&format!("/api/tasks/{task_id}/skills"), json!({ "skills": ["two words"] })).await.status, 400);
    assert_eq!(app.put("/api/tasks/00000000-0000-4000-8000-000000000000/skills", json!({ "skills": [] })).await.status, 404);
}

#[tokio::test]
async fn deleting_a_step_or_a_tag_deletes_its_skills() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let tag = id(&app.post("/api/tags", json!({ "name": "Backend", "icon": "gear", "color": "teal" })).await);
    let task_id = app.a_task(&project_id, "report-builder").await;
    let step_id = app.a_step(&task_id, "Design the picker", None).await;
    app.put(&format!("/api/steps/{step_id}/skills"), json!({ "skills": ["a"] })).await;
    app.put(&format!("/api/tags/{tag}/skills"), json!({ "skills": ["b"] })).await;

    app.delete(&format!("/api/steps/{step_id}")).await;
    app.delete(&format!("/api/tags/{tag}")).await;

    let listing = app.get("/api/skill-assignments").await;
    assert!(listing.get("taskSkills").as_array().unwrap().is_empty());
    assert!(listing.get("tagSkills").as_array().unwrap().is_empty());
}
