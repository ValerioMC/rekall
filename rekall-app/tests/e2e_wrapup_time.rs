//! `RekallEndToEndTest`, part three: the wrapup a session leaves, and the time tracked on a task.

mod support;

use serde_json::{json, Value};
use support::{app, uuid_value};

// --- Wrapup

#[tokio::test]
async fn a_wrapup_written_over_mcp_arrives_with_the_next_context_load() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    app.a_task(&project_id, "report-builder").await;

    let written = app
        .call_tool(
            "rekall_wrapup",
            json!({ "anchors": "project:vega task:report-builder", "body": "## Stato\n\nIl builder legge da POST /api/v1/pipelines." }),
        )
        .await;

    assert_contains!(written, "Wrapup written for", "project:vega task:report-builder");
    assert_contains!(
        app.call_tool("rekall_context", json!({ "anchors": "project:vega task:report-builder" })).await,
        "<wrapup written-by=\"CLAUDE\"",
        "Il builder legge da POST /api/v1/pipelines."
    );
}

#[tokio::test]
async fn writing_a_second_wrapup_replaces_the_first_rather_than_adding_one() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;

    app.call_tool("rekall_wrapup", json!({ "anchors": "project:vega task:report-builder", "body": "Il primo stato." })).await;
    let second = app
        .call_tool("rekall_wrapup", json!({ "anchors": "project:vega task:report-builder", "body": "Lo stato corrente." }))
        .await;

    assert_contains!(second, "Wrapup replaced for");
    assert_eq!(app.count("SELECT COUNT(*) FROM wrapup WHERE task_id = ?", vec![uuid_value(&task_id)]).await, 1);
    let context = app.call_tool("rekall_context", json!({ "anchors": "task:report-builder" })).await;
    assert_contains!(context, "Lo stato corrente.");
    assert_lacks!(context, "Il primo stato.");
}

#[tokio::test]
async fn replacing_a_hand_written_wrapup_says_so_and_the_author_follows_the_last_writer() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;

    let by_hand = app.put(&format!("/api/tasks/{task_id}/wrapup"), json!({ "bodyMarkdown": "Scritto a mano." })).await;

    assert_eq!(by_hand.status, 200);
    assert_eq!(by_hand.str("writtenBy"), "HAND");
    assert_eq!(by_hand.str("anchor"), "project:vega task:report-builder");
    assert_contains!(
        app.call_tool("rekall_wrapup", json!({ "anchors": "project:vega task:report-builder", "body": "Riscritto da Claude." })).await,
        "had been edited by hand"
    );
    assert_eq!(app.get(&format!("/api/tasks/{task_id}/wrapup")).await.str("writtenBy"), "CLAUDE");
}

#[tokio::test]
async fn a_wrapup_refuses_any_anchor_that_does_not_name_exactly_one_task() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let vega = app.a_project(&acme, "vega", "ACTIVE").await;
    let beacon = app.a_project(&acme, "beacon", "ACTIVE").await;
    app.a_task(&vega, "setup").await;
    app.a_task(&beacon, "setup").await;

    assert_contains!(
        app.call_tool("rekall_wrapup", json!({ "anchors": "project:vega", "body": "x" })).await,
        "This write belongs to exactly one task"
    );
    assert_contains!(
        app.call_tool("rekall_wrapup", json!({ "anchors": "task:setup", "body": "x" })).await,
        "matches 2 records",
        "Qualify it with `project:"
    );
    assert_contains!(
        app.call_tool("rekall_wrapup", json!({ "anchors": "task:nowhere", "body": "x" })).await,
        "No task matches 'nowhere'"
    );
    assert_eq!(app.count("SELECT COUNT(*) FROM wrapup", vec![]).await, 0, "nothing was written on any of those");
}

#[tokio::test]
async fn a_wrapup_that_has_grown_into_a_log_is_refused_and_told_why() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    app.a_task(&project_id, "report-builder").await;

    assert_contains!(
        app.call_tool("rekall_wrapup", json!({ "anchors": "project:vega task:report-builder", "body": "x".repeat(20_001) })).await,
        "capped at 20000 characters",
        "not how it got there"
    );
    assert_contains!(
        app.call_tool("rekall_wrapup", json!({ "anchors": "project:vega task:report-builder", "body": "   " })).await,
        "'body' is required"
    );
    assert_eq!(app.count("SELECT COUNT(*) FROM wrapup", vec![]).await, 0);
}

#[tokio::test]
async fn deleting_a_task_takes_its_wrapup_and_deleting_a_wrapup_leaves_the_task() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    app.call_tool("rekall_wrapup", json!({ "anchors": "project:vega task:report-builder", "body": "Lo stato." })).await;

    app.delete(&format!("/api/tasks/{task_id}/wrapup")).await;

    assert_eq!(app.count("SELECT COUNT(*) FROM wrapup", vec![]).await, 0);
    assert_eq!(app.count("SELECT COUNT(*) FROM task", vec![]).await, 1, "the task and its notes are untouched");

    app.call_tool("rekall_wrapup", json!({ "anchors": "project:vega task:report-builder", "body": "Di nuovo." })).await;
    app.delete(&format!("/api/tasks/{task_id}")).await;

    assert_eq!(app.count("SELECT COUNT(*) FROM wrapup", vec![]).await, 0, "and the row goes with the task it describes");
}

#[tokio::test]
async fn a_task_reports_whether_it_has_a_wrapup_without_carrying_the_body() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    app.a_task(&project_id, "report-builder").await;
    app.a_task(&project_id, "retry-policy").await;
    app.call_tool("rekall_wrapup", json!({ "anchors": "project:vega task:report-builder", "body": "Lo stato." })).await;

    let tasks = app.get("/api/tasks").await.list();

    assert_eq!(tasks.len(), 2);
    let mut flags: Vec<String> = tasks.iter().map(|t| format!("{}={}", t["label"].as_str().unwrap(), t["hasWrapup"])).collect();
    flags.sort();
    assert_eq!(flags, ["report-builder=true", "retry-policy=false"]);
}

// --- Time entries

fn entry_with<'a>(entries: &'a [Value], id: &Value) -> &'a Value {
    entries.iter().find(|entry| &entry["id"] == id).expect("the entry is listed")
}

#[tokio::test]
async fn starting_a_timer_opens_a_session_and_stopping_closes_it() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;

    let started = app.post(&format!("/api/tasks/{task_id}/time-entries/start"), json!({})).await;
    assert_eq!(started.str("taskId"), task_id);
    assert!(started.get("stoppedAt").is_null());

    let stopped = app.post(&format!("/api/tasks/{task_id}/time-entries/stop"), json!({})).await;
    assert_eq!(stopped.get("id"), started.get("id"));
    assert!(!stopped.get("stoppedAt").is_null());
}

#[tokio::test]
async fn starting_a_second_tasks_timer_leaves_the_first_one_running() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_a = app.a_task(&project_id, "task-a").await;
    let task_b = app.a_task(&project_id, "task-b").await;

    let first_started = app.post(&format!("/api/tasks/{task_a}/time-entries/start"), json!({})).await;
    let second_started = app.post(&format!("/api/tasks/{task_b}/time-entries/start"), json!({})).await;

    assert_eq!(second_started.str("taskId"), task_b);
    assert!(second_started.get("stoppedAt").is_null());
    let entries = app.get("/api/time-entries").await.list();
    assert!(entry_with(&entries, first_started.get("id"))["stoppedAt"].is_null());
}

#[tokio::test]
async fn starting_a_timer_that_is_already_running_on_this_task_is_a_no_op() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;

    let first = app.post(&format!("/api/tasks/{task_id}/time-entries/start"), json!({})).await;
    let again = app.post(&format!("/api/tasks/{task_id}/time-entries/start"), json!({})).await;

    assert_eq!(again.get("id"), first.get("id"));
    assert_eq!(app.count("SELECT COUNT(*) FROM time_entry WHERE task_id = ?", vec![uuid_value(&task_id)]).await, 1);
}

#[tokio::test]
async fn stopping_a_task_that_is_not_being_tracked_is_refused() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;

    assert_eq!(app.post(&format!("/api/tasks/{task_id}/time-entries/stop"), json!({})).await.status, 400);
}

#[tokio::test]
async fn moving_a_task_to_done_stops_the_session_running_on_it() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    let started = app.post(&format!("/api/tasks/{task_id}/time-entries/start"), json!({})).await;

    app.put(
        &format!("/api/tasks/{task_id}"),
        json!({ "label": "report-builder", "title": "report-builder", "status": "DONE", "projectId": project_id }),
    )
    .await;

    let entries = app.get("/api/time-entries").await.list();
    assert!(!entry_with(&entries, started.get("id"))["stoppedAt"].is_null());
}

#[tokio::test]
async fn moving_a_task_to_a_status_other_than_done_leaves_its_timer_running() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    app.post(&format!("/api/tasks/{task_id}/time-entries/start"), json!({})).await;

    app.put(
        &format!("/api/tasks/{task_id}"),
        json!({ "label": "report-builder", "title": "report-builder", "status": "BLOCKED", "projectId": project_id }),
    )
    .await;

    let entries = app.get("/api/time-entries").await.list();
    assert!(entries.iter().all(|entry| entry["stoppedAt"].is_null()));
}

#[tokio::test]
async fn moving_a_task_to_done_with_nothing_running_is_fine() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;

    let moved = app
        .put(
            &format!("/api/tasks/{task_id}"),
            json!({ "label": "report-builder", "title": "report-builder", "status": "DONE", "projectId": project_id }),
        )
        .await;
    assert_eq!(moved.status, 200);
}

#[tokio::test]
async fn a_session_can_be_corrected_by_hand_within_the_rules_that_keep_it_sane() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    let started = app.post(&format!("/api/tasks/{task_id}/time-entries/start"), json!({})).await;
    let stopped = app.post(&format!("/api/tasks/{task_id}/time-entries/stop"), json!({})).await;
    let entry_id = stopped.str("id");

    let backwards = app
        .patch(
            &format!("/api/time-entries/{entry_id}"),
            json!({ "startedAt": stopped.get("stoppedAt"), "stoppedAt": started.get("startedAt") }),
        )
        .await;
    assert_eq!(backwards.status, 400, "a session has to end after it starts");

    let reopened = app.patch(&format!("/api/time-entries/{entry_id}"), json!({ "startedAt": started.get("startedAt") })).await;
    assert_eq!(reopened.status, 400, "a finished session cannot be reopened");

    let corrected = app
        .patch(
            &format!("/api/time-entries/{entry_id}"),
            json!({ "startedAt": started.get("startedAt"), "stoppedAt": stopped.get("stoppedAt") }),
        )
        .await;
    assert_eq!(corrected.status, 200);
    assert_eq!(corrected.get("startedAt"), started.get("startedAt"));
}

#[tokio::test]
async fn deleting_a_session_removes_only_that_one() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    app.post(&format!("/api/tasks/{task_id}/time-entries/start"), json!({})).await;
    let first = app.post(&format!("/api/tasks/{task_id}/time-entries/stop"), json!({})).await.str("id");
    app.post(&format!("/api/tasks/{task_id}/time-entries/start"), json!({})).await;
    let second = app.post(&format!("/api/tasks/{task_id}/time-entries/stop"), json!({})).await.str("id");

    assert_eq!(app.delete(&format!("/api/time-entries/{first}")).await.status, 204);

    assert_eq!(
        app.count("SELECT COUNT(*) FROM time_entry WHERE task_id = ?", vec![uuid_value(&task_id)]).await,
        1,
        "the other session on the same task survives"
    );
    let remaining: Vec<String> =
        app.get("/api/time-entries").await.list().iter().map(|e| e["id"].as_str().unwrap().to_string()).collect();
    assert_eq!(remaining, [second]);
}

#[tokio::test]
async fn deleting_a_task_takes_its_time_entries_with_it() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    app.post(&format!("/api/tasks/{task_id}/time-entries/start"), json!({})).await;
    app.post(&format!("/api/tasks/{task_id}/time-entries/stop"), json!({})).await;

    app.delete(&format!("/api/tasks/{task_id}")).await;

    assert_eq!(app.count("SELECT COUNT(*) FROM time_entry", vec![]).await, 0);
}
