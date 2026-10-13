//! Steps as a flow, end to end: routes drawn in the console, a session following them over MCP,
//! the context it reads back, and the console's reset.

mod support;

use serde_json::json;
use support::{app, App};

const TASK: &str = "project:vega task:release";

/// Build, deploy, test, with a failed test routed to a fix that routes back to deploy.
async fn release(app: &App) -> (String, [String; 4]) {
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "release").await;
    let build = app.a_step(&task_id, "Build", Some("cargo build --release")).await;
    let deploy = app.a_step(&task_id, "Deploy", Some("helm upgrade")).await;
    let test = app.a_step(&task_id, "Test in the cluster", Some("Run the smoke suite.")).await;
    let fix = app.a_step(&task_id, "Fix the application", Some("Read the failing case.")).await;
    for (from, to, outcome) in [(&build, &deploy, "built"), (&deploy, &test, "deployed"), (&test, &fix, "App error"), (&fix, &deploy, "fixed")] {
        let answer = app.post(&format!("/api/steps/{from}/links"), json!({ "toStepId": to, "outcome": outcome })).await;
        assert_eq!(answer.status, 201, "{}", answer.text);
    }
    (task_id, [build, deploy, test, fix])
}

async fn step(app: &App, step: &str, state: &str, outcome: Option<&str>) -> String {
    let mut arguments = json!({ "anchors": TASK, "step": step, "state": state });
    if let Some(outcome) = outcome {
        arguments["outcome"] = json!(outcome);
    }
    app.call_tool("rekall_step", arguments).await
}

#[tokio::test]
async fn a_route_drawn_in_the_console_comes_back_on_every_step_view() {
    let app = app().await;
    let (task_id, [_, _, test, fix]) = release(&app).await;

    let steps = app.get(&format!("/api/tasks/{task_id}/steps")).await.list();
    let test_view = steps.iter().find(|candidate| candidate["id"] == json!(test)).unwrap();

    assert_eq!(test_view["links"][0]["outcome"], json!("app-error"), "the outcome is normalised to a slug");
    assert_eq!(test_view["links"][0]["toStepId"], json!(fix));
}

#[tokio::test]
async fn a_session_follows_the_routes_and_reads_where_it_stands() {
    let app = app().await;
    release(&app).await;

    assert_lacks!(step(&app, "1", "running", None).await, "Next open step");
    assert_contains!(step(&app, "1", "claimed", None).await, "Routed by `built` to step 2 \"Deploy\"", "step:2 start");
    step(&app, "2", "claimed", None).await;
    assert_contains!(
        step(&app, "Test in the cluster", "claimed", Some("app-error")).await,
        "Routed by `app-error` to step 4 \"Fix the application\""
    );
    assert_contains!(step(&app, "4", "claimed", None).await, "to step 2 \"Deploy\", now open and current (visit 2)");

    let context = app.call_tool("rekall_context", json!({ "anchors": TASK })).await;
    assert_contains!(
        context,
        "flow=\"true\" current=\"2\" moves=\"5\"",
        "- [ ] Deploy  (current, visit 2)",
        "helm upgrade",
        "  Routes: `app-error` -> step 4 \"Fix the application\"",
        "- `flow`: the steps are routed; the work stands on step 2 \"Deploy\" after 5 moves"
    );
}

#[tokio::test]
async fn a_claim_that_names_no_route_is_refused_with_the_routes_there_are() {
    let app = app().await;
    let (task_id, [build, ..]) = release(&app).await;
    let skip = app.a_step(&task_id, "Skip", None).await;
    app.post(&format!("/api/steps/{build}/links"), json!({ "toStepId": skip, "outcome": "skipped" })).await;

    assert_contains!(step(&app, "1", "claimed", None).await, "routes by `built`, `skipped`");
    assert_contains!(step(&app, "1", "claimed", Some("flaky")).await, "has no route for `flaky`");
}

#[tokio::test]
async fn the_console_edits_and_removes_routes_and_resets_the_visits() {
    let app = app().await;
    let (task_id, [build, deploy, test, _]) = release(&app).await;
    step(&app, "1", "claimed", None).await;
    let steps = app.get(&format!("/api/tasks/{task_id}/steps")).await.list();
    let route = steps.iter().find(|candidate| candidate["id"] == json!(build)).unwrap()["links"][0]["id"].as_str().unwrap().to_string();

    let edited = app.patch(&format!("/api/step-links/{route}"), json!({ "toStepId": test, "outcome": "Fast path" })).await;
    assert_eq!(edited.status, 200, "{}", edited.text);
    let taken = app.post(&format!("/api/steps/{build}/links"), json!({ "toStepId": deploy, "outcome": "fast-path" })).await;
    assert_eq!(taken.status, 409, "one outcome names one route: {}", taken.text);
    let onto_itself = app.post(&format!("/api/steps/{build}/links"), json!({ "toStepId": build, "outcome": "again" })).await;
    assert_eq!(onto_itself.status, 400);

    let reset = app.delete(&format!("/api/tasks/{task_id}/visits")).await;
    assert_eq!(reset.status, 200);
    assert!(reset.list().iter().all(|view| view["visits"].as_array().unwrap().is_empty()));

    let removed = app.delete(&format!("/api/step-links/{route}")).await;
    assert_eq!(removed.status, 200);
    assert!(removed.list().iter().find(|view| view["id"] == json!(build)).unwrap()["links"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn a_route_with_no_outcome_is_refused_before_it_reaches_the_service() {
    let app = app().await;
    let (_, [build, deploy, ..]) = release(&app).await;

    let answer = app.post(&format!("/api/steps/{build}/links"), json!({ "toStepId": deploy })).await;

    assert_eq!(answer.status, 400);
}
