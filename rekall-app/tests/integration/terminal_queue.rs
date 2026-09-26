//! `TerminalApiTest` and `RunQueueApiTest`: the in-app terminal and the run queue end to end,
//! against a stub that stands in for the interactive `claude` TUI, with the usage reading set by
//! the test as its `@MockitoBean` did.

mod support;

use std::os::unix::fs::PermissionsExt;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures::{SinkExt, StreamExt};
use rekall_app::StartOptions;
use rekall_claude::usage::{ClaudeUsageView, Limit, Severity, UsageReader};
use rekall_common::Instant;
use serde_json::{json, Value};
use support::{app_with, await_until, id, App};
use tokio_tungstenite::tungstenite::Message;

fn write_stub(folder: &std::path::Path, banner: &str) -> std::path::PathBuf {
    let stub = folder.join(format!("fake-claude-{}.sh", rekall_common::Id::random()));
    std::fs::write(&stub, format!("#!/bin/sh\necho \"{banner}\"\nwhile IFS= read -r line; do\n  echo \"got:$line\"\ndone\n")).unwrap();
    std::fs::set_permissions(&stub, std::fs::Permissions::from_mode(0o755)).unwrap();
    stub
}

async fn close_live_terminals(app: &App) {
    for row in app.get("/api/terminals").await.list() {
        app.delete(&format!("/api/terminals/{}", row["id"].as_str().unwrap())).await;
    }
}

// ---------------------------------------------------------------- TerminalApiTest

const BANNER: &str = "rekall-terminal-stub-ready";

async fn terminal_app(stubs: &tempfile::TempDir) -> App {
    let stub = write_stub(stubs.path(), BANNER);
    app_with(
        &[("rekall.claude.cli-path", stub.to_str().unwrap()), ("rekall.terminal.sweep-minutes", "60")],
        StartOptions { no_restart: true, ..StartOptions::default() },
    )
    .await
}

async fn a_task_with_folder(app: &App, folder: &std::path::Path) -> String {
    let company = app.a_company("Acme").await;
    let project = app.a_project_in(&company, "vega", folder).await;
    id(&app
        .post("/api/tasks", json!({ "label": "report-builder", "title": "Report builder", "status": "IN_PROGRESS", "projectId": project }))
        .await)
}

#[tokio::test]
async fn open_connect_the_pipe_see_the_banner_and_an_echoed_line_close() {
    let stubs = tempfile::tempdir().unwrap();
    let app = terminal_app(&stubs).await;
    let folder = tempfile::tempdir().unwrap();
    let task_id = a_task_with_folder(&app, folder.path()).await;

    let opened = app.post(&format!("/api/tasks/{task_id}/terminals"), json!({ "skipPermissions": true })).await;
    assert_eq!(opened.status, 201);
    let id = opened.str("id");
    assert_eq!(opened.get("live"), &json!(true));
    assert_contains!(opened.str("anchors"), "task:");
    assert_eq!(app.get("/api/terminals").await.list().len(), 1);
    assert_eq!(app.get(&format!("/api/terminals/{id}")).await.str("id"), id);

    let url = format!("ws://localhost:{}/api/terminal/{id}/io", app.running.as_ref().unwrap().port);
    let (mut socket, _) = tokio_tungstenite::connect_async(url).await.unwrap();
    let mut seen = String::new();
    let mut read_until = async |socket: &mut tokio_tungstenite::WebSocketStream<_>, wanted: &str| {
        while !seen.contains(wanted) {
            match tokio::time::timeout(Duration::from_secs(10), socket.next()).await.unwrap().unwrap().unwrap() {
                Message::Binary(bytes) => seen.push_str(&String::from_utf8_lossy(&bytes)),
                Message::Text(_) => {}
                other => panic!("unexpected frame {other:?}"),
            }
        }
    };
    read_until(&mut socket, BANNER).await;
    socket.send(Message::Binary(b"ping\n".to_vec().into())).await.unwrap();
    read_until(&mut socket, "got:ping").await;
    socket.close(None).await.unwrap();

    assert_eq!(app.delete(&format!("/api/terminals/{id}")).await.status, 204);
    assert!(app.get("/api/terminals").await.list().is_empty());
    app.stop().await;
}

#[tokio::test]
async fn a_task_whose_project_has_no_folder_is_refused() {
    let stubs = tempfile::tempdir().unwrap();
    let app = terminal_app(&stubs).await;
    let company = app.a_company("Acme").await;
    let project = app.a_project(&company, "vega", "ACTIVE").await;
    let task = id(&app
        .post("/api/tasks", json!({ "label": "no-folder", "title": "No folder", "status": "TODO", "projectId": project }))
        .await);

    let response = app.post(&format!("/api/tasks/{task}/terminals"), json!({ "skipPermissions": true })).await;

    assert_eq!(response.status, 400);
    assert!(app.get("/api/terminals").await.list().is_empty());
    app.stop().await;
}

#[tokio::test]
async fn connecting_to_an_unknown_terminal_id_closes_the_socket_instead_of_hanging() {
    let stubs = tempfile::tempdir().unwrap();
    let app = terminal_app(&stubs).await;
    let url = format!("ws://localhost:{}/api/terminal/{}/io", app.running.as_ref().unwrap().port, rekall_common::Id::random());
    let (mut socket, _) = tokio_tungstenite::connect_async(url).await.unwrap();

    let closed = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            match socket.next().await {
                None | Some(Err(_)) | Some(Ok(Message::Close(_))) => return,
                Some(Ok(_)) => continue,
            }
        }
    })
    .await;
    assert!(closed.is_ok(), "the socket was closed");
    app.stop().await;
}

// ---------------------------------------------------------------- RunQueueApiTest

struct SetUsage(Mutex<ClaudeUsageView>);

#[async_trait::async_trait]
impl UsageReader for SetUsage {
    async fn current(&self) -> ClaudeUsageView {
        self.0.lock().unwrap().clone()
    }

    async fn refresh(&self) -> ClaudeUsageView {
        self.current().await
    }
}

struct Queue {
    app: App,
    usage: Arc<SetUsage>,
    project_id: String,
    session_reset: Instant,
    _stubs: tempfile::TempDir,
    _folder: tempfile::TempDir,
}

fn reading(session_percent: f64, session_reset: Instant) -> ClaudeUsageView {
    ClaudeUsageView::ok(
        vec![
            Limit { key: "session".into(), label: "Session".into(), percent: session_percent, severity: Severity::Normal, resets_at: Some(session_reset) },
            Limit {
                key: "weekly_all".into(),
                label: "Weekly · all models".into(),
                percent: 5.0,
                severity: Severity::Normal,
                resets_at: Some(session_reset.plus(chrono::TimeDelta::days(3))),
            },
        ],
        Instant::now(),
    )
}

async fn queue_app() -> Queue {
    let stubs = tempfile::tempdir().unwrap();
    let stub = write_stub(stubs.path(), "rekall-queue-stub-ready");
    let session_reset = Instant::now().plus(chrono::TimeDelta::hours(2));
    let usage = Arc::new(SetUsage(Mutex::new(reading(10.0, session_reset))));
    let app = app_with(
        &[
            ("rekall.claude.cli-path", stub.to_str().unwrap()),
            ("rekall.terminal.sweep-minutes", "60"),
            ("rekall.run-queue.tick-seconds", "1"),
            ("rekall.run-queue.settle-grace-seconds", "0"),
        ],
        StartOptions { no_restart: true, usage: Some(usage.clone()), ..StartOptions::default() },
    )
    .await;
    let folder = tempfile::tempdir().unwrap();
    let company = app.a_company("Acme").await;
    let project_id = app.a_project_in(&company, "vega", folder.path()).await;
    Queue { app, usage, project_id, session_reset, _stubs: stubs, _folder: folder }
}

impl Queue {
    fn usage_at(&self, session_percent: f64) {
        *self.usage.0.lock().unwrap() = reading(session_percent, self.session_reset);
    }

    async fn task(&self, label: &str, title: &str) -> String {
        id(&self
            .app
            .post("/api/tasks", json!({ "label": label, "title": title, "status": "IN_PROGRESS", "projectId": self.project_id }))
            .await)
    }

    /// A step as the console leaves it once promoted: open, on the checklist a session reads.
    async fn open_step(&self, task_id: &str, title: &str) {
        self.app.a_step(task_id, title, None).await;
    }

    async fn enqueue(&self, task_id: &str) {
        self.app.post("/api/run-queue/items", json!({ "taskId": task_id })).await;
    }

    async fn queue(&self) -> Value {
        self.app.get("/api/run-queue").await.body
    }

    async fn item(&self, index: usize) -> Value {
        self.queue().await["items"][index].clone()
    }

    async fn item_state(&self, index: usize) -> String {
        self.item(index).await["state"].as_str().unwrap_or("none").to_string()
    }

    async fn queue_state(&self) -> String {
        self.queue().await["state"].as_str().unwrap().to_string()
    }

    async fn terminal_on(&self, task_id: &str) -> Option<String> {
        self.app
            .get("/api/terminals")
            .await
            .list()
            .iter()
            .find(|row| row["taskId"] == task_id)
            .map(|row| row["id"].as_str().unwrap().to_string())
    }

    async fn claim(&self, task_label: &str, step: &str) {
        self.app
            .call_tool("rekall_step", json!({ "anchors": format!("project:vega task:{task_label}"), "step": step, "state": "claimed" }))
            .await;
    }

    async fn tear_down(self) {
        self.app.post("/api/run-queue/stop", json!({})).await;
        close_live_terminals(&self.app).await;
        self.app.stop().await;
    }
}

#[tokio::test]
async fn runs_each_task_in_its_own_terminal_one_after_the_other_and_goes_idle_at_the_end() {
    let q = queue_app().await;
    let checklist = q.task("report-builder", "Report builder").await;
    q.open_step(&checklist, "Build the export").await;
    q.open_step(&checklist, "Expose the download").await;
    let stepless = q.task("fix-readme", "Fix readme").await;
    q.enqueue(&checklist).await;
    q.enqueue(&stepless).await;

    let started = q.app.post("/api/run-queue/start", json!({})).await;
    assert_eq!(started.str("state"), "RUNNING");

    await_until(15, || async { q.item_state(0).await == "RUNNING" && q.terminal_on(&checklist).await.is_some() }).await;
    q.claim("report-builder", "1").await;
    await_until(15, || async { q.item_state(0).await == "RUNNING" }).await;
    assert!(q.terminal_on(&checklist).await.is_some(), "one claim of two leaves the session working");

    q.claim("report-builder", "2").await;
    await_until(15, || async { q.item_state(0).await == "FINISHED" && q.item_state(1).await == "RUNNING" }).await;
    assert!(q.terminal_on(&checklist).await.is_none());
    assert!(q.terminal_on(&stepless).await.is_some());

    // `claimedByWrapup`: what a Claude-written wrapup does to a stepless task.
    q.app.call_tool("rekall_wrapup", json!({ "anchors": "project:vega task:fix-readme", "body": "Done." })).await;
    await_until(15, || async { q.item_state(1).await == "FINISHED" && q.queue_state().await == "IDLE" }).await;
    assert!(q.terminal_on(&stepless).await.is_none());
    q.tear_down().await;
}

#[tokio::test]
async fn a_task_with_nothing_open_is_skipped_and_a_session_that_ends_early_fails_its_item() {
    let q = queue_app().await;
    let claimed = q.task("done-already", "Done already").await;
    q.open_step(&claimed, "Only step").await;
    q.claim("done-already", "1").await;
    let quitter = q.task("quitter", "Quitter").await;
    q.open_step(&quitter, "Never claimed").await;
    q.enqueue(&claimed).await;
    q.enqueue(&quitter).await;

    q.app.post("/api/run-queue/start", json!({})).await;

    await_until(15, || async { q.item_state(0).await == "SKIPPED" && q.item_state(1).await == "RUNNING" }).await;
    let terminal = q.terminal_on(&quitter).await.expect("a terminal on the quitter");
    q.app.delete(&format!("/api/terminals/{terminal}")).await;

    await_until(15, || async { q.item_state(1).await == "FAILED" && q.queue_state().await == "IDLE" }).await;
    let detail = q.item(1).await["detail"].clone();
    assert_contains!(detail.as_str().unwrap(), "ended before");
    q.tear_down().await;
}

#[tokio::test]
async fn over_the_ceiling_at_the_start_the_queue_holds_until_the_session_window_resets() {
    let q = queue_app().await;
    let task = q.task("report-builder", "Report builder").await;
    q.enqueue(&task).await;
    q.app.put("/api/run-queue/settings", json!({ "ceilingPercent": 80, "skipPermissions": true })).await;
    q.usage_at(85.0);

    q.app.post("/api/run-queue/start", json!({})).await;

    await_until(15, || async { q.queue_state().await == "HOLDING" }).await;
    assert!(q.terminal_on(&task).await.is_none());
    assert_eq!(q.item_state(0).await, "QUEUED");
    let queue = q.queue().await;
    let hold_until = Instant::parse(queue["holdUntil"].as_str().unwrap()).unwrap();
    assert!(hold_until.is_after(&q.session_reset));
    assert_contains!(queue["holdReason"].as_str().unwrap(), "85%");
    q.tear_down().await;
}

#[tokio::test]
async fn crossing_the_ceiling_at_a_claim_closes_the_session_and_puts_the_task_back_at_the_head() {
    let q = queue_app().await;
    let task = q.task("report-builder", "Report builder").await;
    q.open_step(&task, "Build the export").await;
    q.open_step(&task, "Expose the download").await;
    q.enqueue(&task).await;
    q.app.put("/api/run-queue/settings", json!({ "ceilingPercent": 80 })).await;

    q.app.post("/api/run-queue/start", json!({})).await;
    await_until(15, || async { q.terminal_on(&task).await.is_some() }).await;

    q.usage_at(82.0);
    q.app
        .call_tool("rekall_step", json!({ "anchors": "project:vega task:report-builder", "step": "2", "state": "running" }))
        .await;
    q.claim("report-builder", "1").await;

    await_until(15, || async { q.queue_state().await == "HOLDING" && q.terminal_on(&task).await.is_none() }).await;
    assert_eq!(q.item_state(0).await, "QUEUED");
    let detail = q.item(0).await["detail"].clone();
    assert_contains!(detail.as_str().unwrap(), "ceiling");
    let steps = q.app.get(&format!("/api/tasks/{task}/steps")).await.list();
    assert_eq!(steps[1]["state"], "OPEN", "a step the session had started is left open for the next one");
    q.tear_down().await;
}

#[tokio::test]
async fn a_start_time_in_the_future_schedules_the_queue_stop_disarms_it() {
    let q = queue_app().await;
    let task = q.task("report-builder", "Report builder").await;
    q.enqueue(&task).await;

    let at = Instant::now().plus(chrono::TimeDelta::hours(3));
    let scheduled = q.app.post("/api/run-queue/start", json!({ "startAt": at.to_string() })).await;

    assert_eq!(scheduled.str("state"), "SCHEDULED");
    assert_eq!(Instant::parse(&scheduled.str("startAt")).unwrap(), at);

    let stopped = q.app.post("/api/run-queue/stop", json!({})).await;
    assert_eq!(stopped.str("state"), "IDLE");
    assert!(stopped.get("startAt").is_null());
    q.tear_down().await;
}

#[tokio::test]
async fn the_queue_refuses_a_duplicate_task_an_empty_start_a_past_time_and_a_silly_ceiling() {
    let q = queue_app().await;
    assert_eq!(q.app.post("/api/run-queue/start", json!({})).await.status, 409);

    let task = q.task("report-builder", "Report builder").await;
    q.enqueue(&task).await;
    assert_eq!(q.app.post("/api/run-queue/items", json!({ "taskId": task })).await.status, 409);

    let past = Instant::now().minus(chrono::TimeDelta::hours(1));
    assert_eq!(q.app.post("/api/run-queue/start", json!({ "startAt": past.to_string() })).await.status, 400);

    assert_eq!(q.app.put("/api/run-queue/settings", json!({ "ceilingPercent": 140 })).await.status, 400);
    q.tear_down().await;
}

#[tokio::test]
async fn waiting_items_reorder_among_themselves_and_settled_ones_clear() {
    let q = queue_app().await;
    let first = q.task("first", "First").await;
    let second = q.task("second", "Second").await;
    q.enqueue(&first).await;
    q.enqueue(&second).await;

    let second_item = q.item(1).await["id"].as_str().unwrap().to_string();
    let moved = q.app.put(&format!("/api/run-queue/items/{second_item}/position"), json!({ "index": 0 })).await;
    assert_eq!(moved.get("items")[0]["taskId"], json!(second));

    q.app
        .execute("UPDATE run_queue_item SET state = 'FINISHED' WHERE task_id = ?", vec![support::uuid_value(&first)])
        .await;
    let cleared = q.app.post("/api/run-queue/clear", json!({})).await;
    assert_eq!(cleared.get("items").as_array().unwrap().len(), 1);
    q.tear_down().await;
}
