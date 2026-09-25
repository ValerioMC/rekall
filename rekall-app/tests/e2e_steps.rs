//! `RekallEndToEndTest`, part four: the checklist a session works through, the review line of a
//! stepless task, and the event stream an open console follows.

mod support;

use serde_json::json;
use support::{app, uuid_value};

#[tokio::test]
async fn open_steps_reach_claude_in_full_done_steps_the_wrapup_covers_by_name_alone() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    let aggregate = app.a_step(&task_id, "Aggregate the rows", Some("Somma per settimana, gruppo per progetto.")).await;
    app.a_step(&task_id, "Write the tests", Some("Un caso per settimana vuota e uno per settimana piena.")).await;

    app.patch(&format!("/api/steps/{aggregate}"), json!({ "done": true })).await;
    // Written after the tick, so the finished step is accounted for.
    app.call_tool("rekall_wrapup", json!({ "anchors": "project:vega task:report-builder", "body": "Le righe sono aggregate." })).await;

    let context = app.call_tool("rekall_context", json!({ "anchors": "task:report-builder" })).await;

    assert_contains!(
        context,
        "- `steps`: 1 of 2 done, 1 open",
        "<steps done=\"1\" open=\"1\">",
        "- [x] Aggregate the rows",
        "- [ ] Write the tests",
        "Un caso per settimana vuota"
    );
    assert_lacks!(context, "Somma per settimana");
}

#[tokio::test]
async fn a_step_finished_after_the_last_wrapup_is_marked_and_gets_its_detail_back() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    let early = app.a_step(&task_id, "Modello e migrazione", Some("Entita Report, changeset Liquibase.")).await;
    let late = app.a_step(&task_id, "Aggregazione delle righe", Some("Somma per settimana, gruppo per progetto.")).await;
    app.a_step(&task_id, "Scrivere i test", Some("Un caso per settimana vuota.")).await;

    app.patch(&format!("/api/steps/{early}"), json!({ "done": true })).await;
    app.call_tool("rekall_wrapup", json!({ "anchors": "project:vega task:report-builder", "body": "Il modello esiste." })).await;
    app.patch(&format!("/api/steps/{late}"), json!({ "done": true })).await;

    let context = app.call_tool("rekall_context", json!({ "anchors": "task:report-builder" })).await;
    assert_contains!(
        context,
        "finished-since-wrapup=\"1\"",
        "- [x] Aggregazione delle righe  (finished since the wrapup was written)",
        "Somma per settimana, gruppo per progetto.",
        "- [x] Modello e migrazione\n"
    );
    assert_lacks!(context, "changeset Liquibase");

    app.call_tool(
        "rekall_wrapup",
        json!({ "anchors": "project:vega task:report-builder", "body": "Il modello esiste e le righe sono aggregate per settimana." }),
    )
    .await;

    let context = app.call_tool("rekall_context", json!({ "anchors": "task:report-builder" })).await;
    assert_lacks!(context, "finished-since-wrapup", "finished since the wrapup was written", "Somma per settimana, gruppo per progetto.");
}

#[tokio::test]
async fn with_no_wrapup_yet_every_finished_step_is_marked_and_carries_its_detail() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    let first = app.a_step(&task_id, "Modello", Some("Entita Report.")).await;
    let second = app.a_step(&task_id, "Endpoint", Some("POST /api/v1/reports.")).await;
    app.patch(&format!("/api/steps/{first}"), json!({ "done": true })).await;
    app.patch(&format!("/api/steps/{second}"), json!({ "done": true })).await;

    assert_contains!(
        app.call_tool("rekall_context", json!({ "anchors": "task:report-builder" })).await,
        "finished-since-wrapup=\"2\"",
        "Entita Report.",
        "POST /api/v1/reports."
    );
}

#[tokio::test]
async fn a_task_with_no_steps_carries_no_steps_block() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    app.a_task(&project_id, "report-builder").await;

    assert_lacks!(app.call_tool("rekall_context", json!({ "anchors": "task:report-builder" })).await, "<steps", "`steps`");
}

#[tokio::test]
async fn a_draft_step_is_the_creation_default_is_kept_off_the_sessions_checklist_and_promotes_to_open() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;

    let created = app
        .post(&format!("/api/tasks/{task_id}/steps"), json!({ "title": "Aggregate the rows", "bodyMarkdown": "Somma per settimana." }))
        .await;
    assert_eq!(created.str("state"), "DRAFT", "a step is born a draft");
    let step_id = created.str("id");

    let context = app.call_tool("rekall_context", json!({ "anchors": "task:report-builder" })).await;
    assert_lacks!(context, "<steps", "Somma per settimana");
    assert_contains!(context, "- `drafts`: 1 not yet promoted to the checklist");

    assert_contains!(
        app.call_tool("rekall_step", json!({ "anchors": "project:vega task:report-builder", "step": "1", "state": "running" })).await,
        "still a draft"
    );

    app.patch(&format!("/api/steps/{step_id}"), json!({ "draft": false })).await;

    let context = app.call_tool("rekall_context", json!({ "anchors": "task:report-builder" })).await;
    assert_contains!(context, "<steps done=\"0\" open=\"1\">", "- [ ] Aggregate the rows", "Somma per settimana.");
    assert_lacks!(context, "`drafts`");
}

#[tokio::test]
async fn an_open_step_returns_to_draft_a_claimed_one_does_not() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    let step_id = app.a_step(&task_id, "Aggregate the rows", None).await;

    let back_to_draft = app.patch(&format!("/api/steps/{step_id}"), json!({ "draft": true })).await;
    assert_eq!(back_to_draft.str("state"), "DRAFT");

    app.patch(&format!("/api/steps/{step_id}"), json!({ "draft": false })).await;
    app.call_tool("rekall_step", json!({ "anchors": "project:vega task:report-builder", "step": "1", "state": "claimed" })).await;

    let refused = app.patch(&format!("/api/steps/{step_id}"), json!({ "draft": true })).await;
    assert!((400..500).contains(&refused.status));
}

#[tokio::test]
async fn steps_are_appended_reordered_and_renumbered_when_one_is_removed() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    let first = app.a_step(&task_id, "First", None).await;
    app.a_step(&task_id, "Second", None).await;
    let third = app.a_step(&task_id, "Third", None).await;

    assert_eq!(app.labels_of_steps(&task_id).await, ["First", "Second", "Third"]);

    let reordered = app.post(&format!("/api/steps/{third}/move"), json!({ "position": 0 })).await.list();
    let titles: Vec<&str> = reordered.iter().map(|s| s["title"].as_str().unwrap()).collect();
    assert_eq!(titles, ["Third", "First", "Second"]);

    app.delete(&format!("/api/steps/{first}")).await;

    assert_eq!(app.labels_of_steps(&task_id).await, ["Third", "Second"]);
    let positions: Vec<i64> = app
        .query("SELECT position FROM task_step WHERE task_id = ? ORDER BY position", vec![uuid_value(&task_id)])
        .await
        .iter()
        .map(|row| row.try_get_by_index::<i64>(0).unwrap())
        .collect();
    assert_eq!(positions, [0, 1], "dense from zero, with no gap where the deleted one was");
}

#[tokio::test]
async fn moving_a_step_past_the_end_of_the_list_puts_it_at_the_end() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    let first = app.a_step(&task_id, "First", None).await;
    app.a_step(&task_id, "Second", None).await;

    app.post(&format!("/api/steps/{first}/move"), json!({ "position": 99 })).await;

    assert_eq!(app.labels_of_steps(&task_id).await, ["Second", "First"]);
}

#[tokio::test]
async fn a_step_is_ticked_without_carrying_its_title_or_its_detail() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    let step_id = app.a_step(&task_id, "Write the tests", Some("Un caso per settimana vuota.")).await;

    let ticked = app.patch(&format!("/api/steps/{step_id}"), json!({ "done": true })).await;
    assert_eq!(ticked.get("done"), &json!(true));
    assert_eq!(ticked.str("title"), "Write the tests");
    assert_eq!(ticked.str("bodyMarkdown"), "Un caso per settimana vuota.");
    assert!(!ticked.get("doneAt").is_null(), "the flag and the moment are one fact");

    let reopened = app.patch(&format!("/api/steps/{step_id}"), json!({ "done": false })).await;
    assert_eq!(reopened.get("done"), &json!(false));
    assert!(reopened.get("doneAt").is_null(), "and it is cleared again when it reopens");
}

#[tokio::test]
async fn a_task_row_reports_how_much_of_its_checklist_is_done() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    app.a_task(&project_id, "retry-policy").await;
    let first = app.a_step(&task_id, "First", None).await;
    app.a_step(&task_id, "Second", None).await;
    app.patch(&format!("/api/steps/{first}"), json!({ "done": true })).await;

    let mut rows: Vec<String> = app
        .get("/api/tasks")
        .await
        .list()
        .iter()
        .map(|t| format!("{}={}/{}", t["label"].as_str().unwrap(), t["stepsDone"], t["stepCount"]))
        .collect();
    rows.sort();
    assert_eq!(rows, ["report-builder=1/2", "retry-policy=0/0"]);
}

#[tokio::test]
async fn deleting_a_task_takes_its_checklist_with_it() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    app.a_step(&task_id, "First", None).await;
    app.a_step(&task_id, "Second", None).await;

    app.delete(&format!("/api/tasks/{task_id}")).await;

    assert_eq!(app.count("SELECT COUNT(*) FROM task_step", vec![]).await, 0);
}

#[tokio::test]
async fn a_step_with_no_title_or_with_a_detail_the_size_of_a_document_is_refused() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;

    let untitled = app.post(&format!("/api/tasks/{task_id}/steps"), json!({ "title": "   ", "bodyMarkdown": null })).await;
    assert_eq!(untitled.status, 400);

    let too_much = app
        .post(&format!("/api/tasks/{task_id}/steps"), json!({ "title": "Too much", "bodyMarkdown": "x".repeat(20_001) }))
        .await;
    assert_contains!(too_much.detail(), "capped at 20000 characters", "a task of its own");

    assert_eq!(app.count("SELECT COUNT(*) FROM task_step", vec![]).await, 0);
}

// --- Live steps

#[tokio::test]
async fn marking_a_step_running_shows_it_as_in_progress_on_the_next_load() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    app.a_step(&task_id, "Aggregate the rows", Some("Somma per settimana.")).await;
    app.a_step(&task_id, "Write the tests", None).await;

    assert_contains!(
        app.call_tool("rekall_step", json!({ "anchors": "project:vega task:report-builder", "step": "1", "state": "running" })).await,
        "is now `running`",
        "Next open step: 2 \"Write the tests\""
    );
    assert_contains!(
        app.call_tool("rekall_context", json!({ "anchors": "task:report-builder" })).await,
        "<steps done=\"0\" open=\"2\" running=\"1\">",
        "- [ ] Aggregate the rows  (in progress)",
        "Somma per settimana."
    );
}

#[tokio::test]
async fn a_claimed_step_is_finished_work_waiting_for_the_console_to_accept_it() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    let first = app.a_step(&task_id, "Aggregate the rows", Some("Somma per settimana.")).await;
    app.a_step(&task_id, "Write the tests", None).await;

    app.call_tool("rekall_step", json!({ "anchors": "project:vega task:report-builder", "step": "1", "state": "running" })).await;
    assert_contains!(
        app.call_tool("rekall_step", json!({ "anchors": "project:vega task:report-builder", "step": "1", "state": "claimed" })).await,
        "is now `claimed`",
        "Write the wrapup"
    );
    app.call_tool("rekall_wrapup", json!({ "anchors": "project:vega task:report-builder", "body": "Le righe sono aggregate." })).await;

    assert_contains!(
        app.call_tool("rekall_context", json!({ "anchors": "task:report-builder" })).await,
        "<steps done=\"1\" open=\"1\" awaiting-review=\"1\">",
        "- [x] Aggregate the rows  (claimed, waiting for the console to accept it)"
    );

    let rows: Vec<String> =
        app.get("/api/tasks").await.list().iter().map(|t| format!("{}={}", t["label"].as_str().unwrap(), t["stepsDone"])).collect();
    assert_eq!(rows, ["report-builder=0"]);

    app.patch(&format!("/api/steps/{first}"), json!({ "done": true })).await;
    let context = app.call_tool("rekall_context", json!({ "anchors": "task:report-builder" })).await;
    assert_contains!(context, "<steps done=\"1\" open=\"1\">");
    assert_lacks!(context, "awaiting-review");
}

#[tokio::test]
async fn a_session_cannot_mark_a_step_done_only_claimed() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    app.a_step(&task_id, "Aggregate the rows", None).await;

    assert_contains!(
        app.call_tool("rekall_step", json!({ "anchors": "project:vega task:report-builder", "step": "1", "state": "done" })).await,
        "Marked `claimed`, not done",
        "cannot tick the last box"
    );
    assert_contains!(
        app.call_tool("rekall_context", json!({ "anchors": "task:report-builder" })).await,
        "awaiting-review=\"1\"",
        "(claimed, waiting for the console to accept it)"
    );
    assert_eq!(app.get("/api/tasks").await.list()[0]["stepsDone"], json!(0));
}

#[tokio::test]
async fn a_step_reference_that_matches_nothing_is_refused_with_the_list() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    app.a_step(&task_id, "Aggregate the rows", None).await;

    assert_contains!(
        app.call_tool("rekall_step", json!({ "anchors": "project:vega task:report-builder", "step": "7", "state": "running" })).await,
        "There is no step 7"
    );
    assert_contains!(
        app.call_tool("rekall_step", json!({ "anchors": "project:vega task:report-builder", "step": "nope", "state": "running" })).await,
        "No step matches 'nope'",
        "1. Aggregate the rows"
    );
}

#[tokio::test]
async fn a_step_claimed_before_the_wrapup_stays_covered_when_the_console_ticks_it_later() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    let first = app.a_step(&task_id, "Aggregate the rows", Some("Somma per settimana.")).await;
    app.a_step(&task_id, "Write the tests", None).await;

    app.call_tool("rekall_step", json!({ "anchors": "project:vega task:report-builder", "step": "1", "state": "claimed" })).await;
    app.call_tool("rekall_wrapup", json!({ "anchors": "project:vega task:report-builder", "body": "Le righe sono aggregate." })).await;
    app.patch(&format!("/api/steps/{first}"), json!({ "done": true })).await;

    assert_lacks!(
        app.call_tool("rekall_context", json!({ "anchors": "task:report-builder" })).await,
        "finished-since-wrapup",
        "finished since the wrapup was written",
        "Somma per settimana."
    );
}

#[tokio::test]
async fn a_step_moved_over_mcp_reaches_an_open_console_over_the_event_stream() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    app.a_step(&task_id, "Aggregate the rows", None).await;

    let mut lines = app.open_stream().await;
    lines.await_line("event:open", 5).await;

    app.call_tool("rekall_step", json!({ "anchors": "project:vega task:report-builder", "step": "1", "state": "running" })).await;

    assert_eq!(lines.await_line("event:steps", 5).await, "event:steps");
    let data = lines.await_line("data:", 5).await;
    assert_contains!(data, &task_id, "\"state\":\"RUNNING\"");
}

// --- Task-scoped review line (stepless tasks)

#[tokio::test]
async fn a_claude_authored_wrapup_claims_a_task_that_has_no_checklist() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;

    app.call_tool("rekall_wrapup", json!({ "anchors": "project:vega task:report-builder", "body": "Lo stato corrente." })).await;

    let task = app.get_task(&task_id).await;
    assert_eq!(task["reviewActive"], json!(true));
    assert_eq!(task["reviewState"], "CLAIMED");
    assert!(!task["claimedAt"].is_null());
}

#[tokio::test]
async fn a_hand_written_wrapup_is_the_reviewers_correction_so_it_does_not_claim_the_task() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;

    app.put(&format!("/api/tasks/{task_id}/wrapup"), json!({ "bodyMarkdown": "Scritto a mano." })).await;

    assert_eq!(app.get_task(&task_id).await["reviewState"], "OPEN");
}

#[tokio::test]
async fn the_console_accepts_a_stepless_task_and_cannot_accept_it_twice() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;

    let accepted = app.review(&task_id, json!({ "reviewState": "DONE" })).await;
    assert_eq!(accepted.status, 200);
    assert_eq!(accepted.str("reviewState"), "DONE");
    assert!(!accepted.get("acceptedAt").is_null());

    assert_eq!(app.review(&task_id, json!({ "reviewState": "DONE" })).await.status, 400, "a second accept has nothing to do and says so");
}

#[tokio::test]
async fn sending_a_stepless_task_back_reopens_it_and_carries_a_note_to_the_next_session() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    app.call_tool("rekall_wrapup", json!({ "anchors": "project:vega task:report-builder", "body": "Lo stato." })).await;

    let sent_back = app.review(&task_id, json!({ "reviewState": "OPEN", "note": "la colonna export e' ancora sbagliata" })).await;

    assert_eq!(sent_back.status, 200);
    assert_eq!(sent_back.str("reviewState"), "OPEN");
    assert_eq!(sent_back.str("reviewNote"), "la colonna export e' ancora sbagliata");
    assert!(sent_back.get("claimedAt").is_null());
    assert_contains!(
        app.call_tool("rekall_context", json!({ "anchors": "project:vega task:report-builder" })).await,
        "la colonna export e' ancora sbagliata"
    );
}

#[tokio::test]
async fn adding_a_checklist_retires_the_task_level_review_line() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    app.call_tool("rekall_wrapup", json!({ "anchors": "project:vega task:report-builder", "body": "Lo stato." })).await;

    app.a_step(&task_id, "Aggregate the rows", None).await;

    assert_eq!(app.get_task(&task_id).await["reviewActive"], json!(false));
    assert_eq!(app.review(&task_id, json!({ "reviewState": "DONE" })).await.status, 400);
}

#[tokio::test]
async fn the_review_endpoint_refuses_the_two_states_nothing_outside_the_system_may_set() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;

    assert_eq!(app.review(&task_id, json!({ "reviewState": "RUNNING" })).await.status, 400);
    assert_eq!(app.review(&task_id, json!({ "reviewState": "CLAIMED" })).await.status, 400);
}

#[tokio::test]
async fn a_stepless_task_accepted_in_the_console_reaches_an_open_console_over_the_event_stream() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;

    let mut lines = app.open_stream().await;
    lines.await_line("event:open", 5).await;
    app.review(&task_id, json!({ "reviewState": "DONE" })).await;

    assert_eq!(lines.await_line("event:task-review", 5).await, "event:task-review");
    assert_contains!(lines.await_line("data:", 5).await, &task_id, "\"reviewState\":\"DONE\"");
}

#[tokio::test]
async fn a_wrapup_written_and_then_deleted_reaches_an_open_console_over_the_event_stream() {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;

    let mut lines = app.open_stream().await;
    lines.await_line("event:open", 5).await;

    app.call_tool("rekall_wrapup", json!({ "anchors": "project:vega task:report-builder", "body": "Lo stato corrente." })).await;

    assert_eq!(lines.await_line("event:wrapup", 5).await, "event:wrapup");
    assert_contains!(lines.await_line("data:", 5).await, &task_id, "Lo stato corrente.", "\"writtenBy\":\"CLAUDE\"", "\"deleted\":false");

    app.delete(&format!("/api/tasks/{task_id}/wrapup")).await;

    assert_eq!(lines.await_line("event:wrapup", 5).await, "event:wrapup");
    assert_contains!(lines.await_line("data:", 5).await, &task_id, "\"deleted\":true");
}
