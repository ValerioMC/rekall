//! The working set over HTTP: tasks a person keeps open, where closing one lands, and the place
//! last seen in a project.

mod support;

use serde_json::json;
use support::app;

fn task_ids(view: &support::Resp) -> Vec<String> {
    view.get("tasks").as_array().unwrap().iter().map(|open| open["taskId"].as_str().unwrap().to_string()).collect()
}

#[tokio::test]
async fn opening_twice_answers_the_same_set_and_closing_answers_the_neighbour() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project = app.a_project(&acme, "vega", "ACTIVE").await;
    let alpha = app.a_task(&project, "alpha").await;
    let beta = app.a_task(&project, "beta").await;

    app.put(&format!("/api/working-set/tasks/{alpha}"), json!({})).await;
    app.put(&format!("/api/working-set/tasks/{beta}"), json!({})).await;
    let again = app.put(&format!("/api/working-set/tasks/{alpha}"), json!({})).await;

    assert_eq!((again.status, task_ids(&again)), (200, vec![alpha.clone(), beta.clone()]));
    assert_eq!(app.get("/api/working-set").await.get("tasks")[0]["projectId"], project.as_str());

    let landing = app.delete(&format!("/api/working-set/tasks/{alpha}")).await;
    assert_eq!((landing.status, landing.str("taskId"), landing.str("projectId")), (200, beta.clone(), project.clone()));
    assert_eq!(task_ids(&app.get("/api/working-set").await), [beta.clone()]);

    let last = app.delete(&format!("/api/working-set/tasks/{beta}")).await;
    assert!(last.get("taskId").is_null(), "the last open task of a project lands on its task list");
}

#[tokio::test]
async fn an_unknown_task_is_404_and_a_malformed_id_is_400() {
    let app = app().await;

    assert_eq!(app.put("/api/working-set/tasks/00000000-0000-4000-8000-000000000000", json!({})).await.status, 404);
    assert_eq!(app.put("/api/working-set/tasks/not-an-id", json!({})).await.status, 400);
}

#[tokio::test]
async fn a_place_is_remembered_per_project_and_a_task_of_another_project_is_400() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let vega = app.a_project(&acme, "vega", "ACTIVE").await;
    let atlas = app.a_project(&acme, "atlas", "ACTIVE").await;
    let alpha = app.a_task(&vega, "alpha").await;
    let delta = app.a_task(&atlas, "delta").await;

    let remembered = app.put(&format!("/api/working-set/places/{vega}"), json!({ "taskId": alpha })).await;
    let refused = app.put(&format!("/api/working-set/places/{vega}"), json!({ "taskId": delta })).await;
    let list = app.put(&format!("/api/working-set/places/{atlas}"), json!({})).await;

    assert_eq!((remembered.status, remembered.str("taskId")), (200, alpha.clone()));
    assert_eq!(refused.status, 400, "{}", refused.text);
    assert!(list.get("taskId").is_null());
    assert_eq!(app.get("/api/working-set").await.get("places").as_array().unwrap().len(), 2);
}
