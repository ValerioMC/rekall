//! The revisions under a task's description, end to end: saving the description writes them, a session answers them.

mod support;

use serde_json::{json, Value};
use support::{app, App};

const ANCHORS: &str = "project:vega task:report-builder";

struct Scene {
    app: App,
    task_id: String,
    project_id: String,
}

async fn scene() -> Scene {
    let app = app().await;
    let acme = app.a_company("Acme").await;
    let project_id = app.a_project(&acme, "vega", "ACTIVE").await;
    let task_id = app.a_task(&project_id, "report-builder").await;
    Scene { app, task_id, project_id }
}

impl Scene {
    async fn save_description(&self, description: &str) {
        let body = json!({ "label": "report-builder", "title": "report-builder", "status": "TODO", "projectId": self.project_id, "description": description });
        assert_eq!(self.app.put(&format!("/api/tasks/{}", self.task_id), body).await.status, 200);
    }

    async fn thread(&self) -> Vec<Value> {
        self.app.get(&format!("/api/tasks/{}/feedback", self.task_id)).await.list()
    }

    async fn revise(&self, description: &str) -> Value {
        self.save_description(description).await;
        self.thread().await.pop().unwrap()
    }

    async fn evaluate_with_a_wrapup(&self) {
        self.app.call_tool("rekall_wrapup", json!({ "anchors": ANCHORS, "body": "The export streams CSV." })).await;
    }

    async fn reply_to(&self, seq: i64) {
        let answer = self.app.call_tool("rekall_feedback", json!({ "anchors": ANCHORS, "replies": [{ "seq": seq, "reply": "Done." }] })).await;
        assert_contains!(answer, "Replied to");
    }
}

#[tokio::test]
async fn saving_a_changed_description_of_an_evaluated_task_shows_a_revision_and_discarding_it_restores_the_text() {
    let scene = scene().await;
    scene.save_description("Export to CSV.").await;
    scene.save_description("Export to CSV before it is evaluated.").await;
    assert!(scene.thread().await.is_empty(), "nothing is recorded before the task has been evaluated");
    scene.save_description("Export to CSV.").await;
    scene.evaluate_with_a_wrapup().await;

    scene.save_description("Export to CSV and XLSX.").await;

    let thread = scene.thread().await;
    assert_eq!(thread.len(), 1);
    assert_eq!(
        (thread[0]["kind"].as_str(), thread[0]["state"].as_str(), thread[0]["quotedMarkdown"].as_str(), thread[0]["bodyMarkdown"].as_str()),
        (Some("IMPLEMENTATION"), Some("PENDING"), Some("Export to CSV."), Some("Export to CSV and XLSX."))
    );

    let id = thread[0]["id"].as_str().unwrap().to_string();
    assert_eq!(scene.app.post_empty(&format!("/api/feedback/{id}/discard")).await.status, 204);

    assert_eq!(scene.app.get_task(&scene.task_id).await["description"], "Export to CSV.");
    assert!(scene.thread().await.is_empty());
}

#[tokio::test]
async fn a_waiting_revision_is_pointed_at_the_plan_or_the_work_and_nothing_else_can_be_sent() {
    let scene = scene().await;
    scene.save_description("Export to CSV.").await;
    scene.evaluate_with_a_wrapup().await;
    let id = scene.revise("Export to CSV and XLSX.").await["id"].as_str().unwrap().to_string();

    let pointed = scene.app.patch(&format!("/api/feedback/{id}"), json!({ "kind": "PLAN" })).await;
    let missing = scene.app.patch(&format!("/api/feedback/{id}"), json!({})).await;
    let unknown = scene.app.patch(&format!("/api/feedback/{id}"), json!({ "kind": "REVISION" })).await;

    assert_eq!((pointed.status, pointed.str("kind")), (200, "PLAN".into()), "{}", pointed.text);
    assert_eq!((missing.status, unknown.status), (400, 400));
    assert_eq!(scene.thread().await[0]["kind"], "PLAN");
}

#[tokio::test]
async fn a_revision_a_session_applied_can_no_longer_be_pointed_elsewhere_or_discarded() {
    let scene = scene().await;
    scene.save_description("Export to CSV.").await;
    scene.evaluate_with_a_wrapup().await;
    let id = scene.revise("Export to CSV and XLSX.").await["id"].as_str().unwrap().to_string();
    scene.reply_to(1).await;

    let pointed = scene.app.patch(&format!("/api/feedback/{id}"), json!({ "kind": "PLAN" })).await;
    let discarded = scene.app.post_empty(&format!("/api/feedback/{id}/discard")).await;

    assert_eq!((pointed.status, discarded.status), (409, 409));
    let thread = scene.thread().await;
    assert_eq!((thread[0]["state"].as_str(), thread[0]["replyMarkdown"].as_str(), thread[0]["kind"].as_str()), (Some("APPLIED"), Some("Done."), Some("IMPLEMENTATION")));
}

#[tokio::test]
async fn a_revision_is_aimed_at_the_plan_when_the_task_has_steps() {
    let scene = scene().await;
    scene.save_description("Export to CSV.").await;
    scene.app.a_step(&scene.task_id, "Aggregate the rows", None).await;
    scene.evaluate_with_a_wrapup().await;

    let revision = scene.revise("Export to CSV and XLSX.").await;

    assert_eq!(revision["kind"], "PLAN");
}

#[tokio::test]
async fn a_change_to_the_revisions_reaches_the_open_console_as_a_feedback_frame() {
    let scene = scene().await;
    scene.save_description("Export to CSV.").await;
    scene.evaluate_with_a_wrapup().await;
    let mut lines = scene.app.open_stream().await;
    lines.await_line("event:open", 5).await;

    scene.save_description("Export to CSV and XLSX.").await;

    assert_eq!(lines.await_line("event:feedback", 5).await, "event:feedback");
    assert_contains!(lines.await_line("data:", 5).await, &scene.task_id);
}

#[tokio::test]
async fn the_whole_loop_from_a_rewrite_to_the_sessions_reply() {
    let scene = scene().await;
    scene.save_description("Export to CSV.").await;
    scene.evaluate_with_a_wrapup().await;

    scene.save_description("Export to CSV and XLSX.").await;

    assert_contains!(
        scene.app.call_tool("rekall_context", json!({ "anchors": ANCHORS })).await,
        "<thread pending=\"1\" running=\"0\" applied=\"0\">",
        "<entry seq=\"1\" kind=\"IMPLEMENTATION\" state=\"PENDING\">",
        "<quoted>\nExport to CSV.\n</quoted>",
        "<new>\nExport to CSV and XLSX.\n</new>"
    );

    let answer = scene.app.call_tool("rekall_feedback", json!({ "anchors": ANCHORS, "replies": [{ "seq": 1, "reply": "Added XLSX." }] })).await;

    assert_contains!(answer, "Replied to #1", "No entry is open any more.");
    assert!(scene.thread().await.iter().all(|entry| entry["state"] == "APPLIED"));
    assert_contains!(
        scene.app.call_tool("rekall_context", json!({ "anchors": ANCHORS })).await,
        "<thread pending=\"0\" running=\"0\" applied=\"1\">",
        "<entry seq=\"1\" kind=\"IMPLEMENTATION\" state=\"APPLIED\">Added XLSX.</entry>"
    );
}

#[tokio::test]
async fn a_pass_takes_the_waiting_revision_a_session_answers_it_and_a_later_rewrite_waits_for_the_next() {
    let scene = scene().await;
    scene.save_description("Export to CSV.").await;
    scene.evaluate_with_a_wrapup().await;
    scene.save_description("Export to CSV and XLSX.").await;
    let pass_path = format!("/api/tasks/{}/feedback/passes", scene.task_id);

    let nothing_yet = scene.app.post_empty(&format!("/api/tasks/{}/feedback/passes", scene.app.a_task(&scene.project_id, "other").await)).await;
    assert_eq!(nothing_yet.status, 400, "a task with nothing waiting has no pass to start");
    let started = scene.app.post_empty(&pass_path).await;

    assert_eq!(started.status, 201, "{}", started.text);
    let taken = started.list();
    assert_eq!(taken.iter().map(|entry| (entry["seq"].as_i64(), entry["state"].as_str(), entry["pass"].as_i64())).collect::<Vec<_>>(), [(Some(1), Some("RUNNING"), Some(1))]);
    scene.save_description("Export to CSV, XLSX and PDF.").await;
    assert_eq!(scene.app.post_empty(&pass_path).await.status, 409, "a pass is still running");
    assert_contains!(
        scene.app.call_tool("rekall_context", json!({ "anchors": ANCHORS })).await,
        "<thread pending=\"1\" running=\"1\" applied=\"0\">",
        "state=\"RUNNING\" pass=\"1\""
    );

    let answer = scene.app.call_tool("rekall_feedback", json!({ "anchors": ANCHORS, "replies": [{ "seq": 1, "reply": "Added XLSX." }] })).await;

    assert_contains!(answer, "Replied to #1", "Still open: #2.");
    let second = scene.app.post_empty(&pass_path).await;
    assert_eq!(second.list().iter().map(|entry| entry["pass"].as_i64()).collect::<Vec<_>>(), [Some(2)]);
}
