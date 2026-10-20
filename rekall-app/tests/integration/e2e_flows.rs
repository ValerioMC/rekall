//! Steps as a flow, end to end: routes drawn in the console, a session following them over MCP,
//! the context it reads back, the console's reset, and a flow a session proposes as drafts.

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
    let claimed = step(&app, "1", "claimed", None).await;
    assert_contains!(claimed, "Routed by `built` to step 2 \"Deploy\"", "step:2 start", "without waiting for anyone");
    assert_lacks!(claimed, "waiting for the console");
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

#[tokio::test]
async fn a_route_to_the_end_finishes_the_flow_for_a_session_and_shows_in_the_context() {
    let app = app().await;
    let (task_id, [_, _, test, _]) = release(&app).await;
    let ended = app.post(&format!("/api/steps/{test}/links"), json!({ "endsFlow": true, "outcome": "Passed" })).await;
    assert_eq!(ended.status, 201, "{}", ended.text);
    let view = ended.list().into_iter().find(|candidate| candidate["id"] == json!(test)).unwrap();
    assert!(view["links"].as_array().unwrap().iter().any(|link| link["toStepId"].is_null() && link["outcome"] == json!("passed")));

    step(&app, "3", "running", None).await;
    let answer = step(&app, "3", "claimed", Some("passed")).await;

    assert_contains!(answer, "Routed by `passed` to the end of the flow", "Write the wrapup for this task now");
    let context = app.call_tool("rekall_context", json!({ "anchors": TASK })).await;
    assert_contains!(context, "`passed` -> end of the flow");
    assert_lacks!(context, " current=\"");
    let steps = app.get(&format!("/api/tasks/{task_id}/steps")).await.list();
    assert!(steps.iter().all(|view| view["visits"].as_array().unwrap().iter().all(|visit| !visit["leftAt"].is_null())));
}

#[tokio::test]
async fn a_step_dragged_on_the_canvas_keeps_its_place_until_the_layout_is_tidied() {
    let app = app().await;
    let (task_id, [build, ..]) = release(&app).await;

    let placed = app.patch(&format!("/api/steps/{build}/flow-place"), json!({ "x": 320, "y": 140 })).await;
    assert_eq!(placed.status, 200, "{}", placed.text);
    let view = placed.list().into_iter().find(|candidate| candidate["id"] == json!(build)).unwrap();
    assert_eq!((view["flowX"].clone(), view["flowY"].clone()), (json!(320), json!(140)));

    let half = app.patch(&format!("/api/steps/{build}/flow-place"), json!({ "x": 10 })).await;
    assert_eq!(half.status, 400, "a place needs both coordinates");
    let far = app.patch(&format!("/api/steps/{build}/flow-place"), json!({ "x": 900000, "y": 0 })).await;
    assert_eq!(far.status, 400);

    let tidied = app.delete(&format!("/api/tasks/{task_id}/flow-places")).await;
    assert_eq!(tidied.status, 200);
    assert!(tidied.list().iter().all(|view| view["flowX"].is_null() && view["flowY"].is_null()));
}

#[tokio::test]
async fn a_session_proposes_a_flow_that_waits_as_routed_drafts_until_the_console_promotes_them() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "release").await;
    let steps = json!([
        { "title": "Deploy", "detail": "helm upgrade", "routes": [{ "outcome": "deployed", "to": 2 }] },
        { "title": "Test in the cluster", "routes": [{ "outcome": "passed", "to": "end" }, { "outcome": "App error", "to": 3 }] },
        { "title": "Fix the application", "routes": [{ "outcome": "fixed", "to": 1 }] }
    ]);

    let answer = app.call_tool("rekall_propose_flow", json!({ "anchors": TASK, "steps": steps })).await;

    assert_contains!(answer, "3 drafts joined by 4 routes");
    let drafts = app.get(&format!("/api/tasks/{task_id}/steps")).await.list();
    assert!(drafts.iter().all(|draft| draft["state"] == json!("DRAFT")));
    assert_eq!(drafts[1]["links"][1]["outcome"], json!("app-error"));
    assert_eq!(drafts[1]["links"][1]["toStepId"], drafts[2]["id"]);
    assert_lacks!(app.call_tool("rekall_context", json!({ "anchors": TASK })).await, "flow=\"true\"", "Deploy");
    assert_contains!(
        app.call_tool("rekall_propose_flow", json!({ "anchors": TASK, "steps": steps })).await,
        "onto a task with no steps"
    );

    for draft in &drafts {
        let promoted = app.patch(&format!("/api/steps/{}", draft["id"].as_str().unwrap()), json!({ "draft": false })).await;
        assert_eq!(promoted.status, 200, "{}", promoted.text);
    }
    assert_contains!(
        app.call_tool("rekall_context", json!({ "anchors": TASK })).await,
        "flow=\"true\"",
        "`app-error` -> step 3 \"Fix the application\""
    );
}

#[tokio::test]
async fn a_proposed_flow_with_a_loop_that_never_ends_is_refused_and_nothing_lands() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "release").await;
    let steps = json!([
        { "title": "Deploy", "routes": [{ "outcome": "deployed", "to": 2 }] },
        { "title": "Test", "routes": [{ "outcome": "app-error", "to": 1 }] }
    ]);

    let answer = app.call_tool("rekall_propose_flow", json!({ "anchors": TASK, "steps": steps })).await;

    assert_contains!(answer, "loop forever through 'Deploy', 'Test'");
    assert!(app.get(&format!("/api/tasks/{task_id}/steps")).await.list().is_empty());
}

#[tokio::test]
async fn a_visit_budget_set_in_the_console_stops_a_session_going_round_the_loop_again() {
    let app = app().await;
    let (_, [_, deploy, _, _]) = release(&app).await;

    let limited = app.patch(&format!("/api/steps/{deploy}/max-visits"), json!({ "maxVisits": 1 })).await;
    assert_eq!(limited.status, 200, "{}", limited.text);
    assert_eq!(app.patch(&format!("/api/steps/{deploy}/max-visits"), json!({ "maxVisits": 0 })).await.status, 400);

    step(&app, "1", "claimed", None).await;
    assert_contains!(app.call_tool("rekall_context", json!({ "anchors": TASK })).await, "- [ ] Deploy  (current, visit 1 of 1)");
    step(&app, "2", "claimed", None).await;
    step(&app, "3", "claimed", Some("app-error")).await;
    assert_contains!(step(&app, "4", "claimed", None).await, "its whole visit budget", "Nothing was moved");

    let lifted = app.patch(&format!("/api/steps/{deploy}/max-visits"), json!({ "maxVisits": null })).await;
    assert_eq!(lifted.status, 200, "{}", lifted.text);
    assert_contains!(step(&app, "4", "claimed", None).await, "to step 2 \"Deploy\", now open and current (visit 2)");
}
