//! The thread under a task's description, end to end: the console writes it, a session answers it.

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

    async fn add(&self, kind: &str, body: &str) -> support::Resp {
        self.app.post(&format!("/api/tasks/{}/feedback", self.task_id), json!({ "kind": kind, "bodyMarkdown": body })).await
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
async fn feedback_is_added_listed_edited_and_deleted_while_it_is_pending() {
    let scene = scene().await;

    let added = scene.add("IMPLEMENTATION", "The totals are wrong.").await;
    assert_eq!(added.status, 201, "{}", added.text);
    assert_eq!((added.get("seq").as_i64(), added.str("kind"), added.str("state")), (Some(1), "IMPLEMENTATION".into(), "PENDING".into()));
    let id = added.str("id");

    let edited = scene.app.patch(&format!("/api/feedback/{id}"), json!({ "bodyMarkdown": "The totals are wrong in the footer." })).await;
    assert_eq!(edited.status, 200, "{}", edited.text);
    assert_eq!(scene.thread().await[0]["bodyMarkdown"], "The totals are wrong in the footer.");

    assert_eq!(scene.app.delete(&format!("/api/feedback/{id}")).await.status, 204);
    assert!(scene.thread().await.is_empty());
}

#[tokio::test]
async fn a_blank_body_a_missing_kind_and_a_typed_revision_are_the_callers_mistake() {
    let scene = scene().await;

    assert_eq!(scene.add("PLAN", "  ").await.status, 400);
    let no_kind = scene.app.post(&format!("/api/tasks/{}/feedback", scene.task_id), json!({ "bodyMarkdown": "x" })).await;
    assert_eq!(no_kind.status, 400);
    assert_eq!(scene.add("REVISION", "x").await.status, 400);
    assert!(scene.thread().await.is_empty());
}

#[tokio::test]
async fn an_entry_a_session_applied_can_no_longer_be_edited_or_deleted() {
    let scene = scene().await;
    let id = scene.add("PLAN", "Split step 2.").await.str("id");
    scene.reply_to(1).await;

    let edit = scene.app.patch(&format!("/api/feedback/{id}"), json!({ "bodyMarkdown": "new" })).await;
    let delete = scene.app.delete(&format!("/api/feedback/{id}")).await;

    assert_eq!((edit.status, delete.status), (409, 409));
    let thread = scene.thread().await;
    assert_eq!((thread[0]["state"].as_str(), thread[0]["replyMarkdown"].as_str(), thread[0]["bodyMarkdown"].as_str()), (Some("APPLIED"), Some("Done."), Some("Split step 2.")));
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
    assert_eq!((thread[0]["kind"].as_str(), thread[0]["quotedMarkdown"].as_str(), thread[0]["bodyMarkdown"].as_str()), (Some("REVISION"), Some("Export to CSV."), Some("Export to CSV and XLSX.")));

    let id = thread[0]["id"].as_str().unwrap().to_string();
    assert_eq!(scene.app.post_empty(&format!("/api/feedback/{id}/discard")).await.status, 204);

    assert_eq!(scene.app.get_task(&scene.task_id).await["description"], "Export to CSV.");
    assert!(scene.thread().await.is_empty());
}

#[tokio::test]
async fn feedback_on_the_work_of_a_claimed_task_reopens_it_and_feedback_on_the_plan_does_not() {
    let scene = scene().await;
    scene.evaluate_with_a_wrapup().await;
    assert_eq!(scene.app.get_task(&scene.task_id).await["reviewState"], "CLAIMED");

    scene.add("PLAN", "Split step 2.").await;
    assert_eq!(scene.app.get_task(&scene.task_id).await["reviewState"], "CLAIMED");

    scene.add("IMPLEMENTATION", "The totals are wrong.").await;
    assert_eq!(scene.app.get_task(&scene.task_id).await["reviewState"], "OPEN");
}

#[tokio::test]
async fn a_change_to_the_thread_reaches_the_open_console_as_a_feedback_frame() {
    let scene = scene().await;
    let mut lines = scene.app.open_stream().await;
    lines.await_line("event:open", 5).await;

    scene.add("PLAN", "Split step 2.").await;

    assert_eq!(lines.await_line("event:feedback", 5).await, "event:feedback");
    assert_contains!(lines.await_line("data:", 5).await, &scene.task_id);
}
