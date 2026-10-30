//! A chat session end to end against a stand-in headless `claude` that replays a recorded
//! conversation: opened through the terminal route with `kind: CHAT`, followed over Server-Sent
//! Events, written to, resumed from the last event seen, and refused what is not its to take.

mod support;

use std::os::unix::fs::PermissionsExt;
use std::time::Duration;

use rekall_app::StartOptions;
use serde_json::{json, Value};
use support::{app_with, App};

const FAKE_CLAUDE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../rekall-infra/tests/fixtures/fake_claude.sh");
const PLAIN_REPLY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../rekall-infra/tests/fixtures/chat/plain_reply.jsonl");

struct Frame {
    id: u64,
    event: Value,
}

/// An open `GET /api/chat/{id}/events`, read one event at a time.
struct EventStream {
    response: reqwest::Response,
    buffer: String,
}

impl EventStream {
    async fn open(app: &App, id: &str, last_event_id: Option<u64>) -> Self {
        let mut request = app.client.get(app.url(&format!("/api/chat/{id}/events")));
        if let Some(last) = last_event_id {
            request = request.header("Last-Event-ID", last.to_string());
        }
        let response = request.send().await.expect("the server answers");
        assert_eq!(response.status(), 200);
        assert_eq!(response.headers()["content-type"], "text/event-stream");
        Self { response, buffer: String::new() }
    }

    async fn next(&mut self) -> Option<Frame> {
        loop {
            if let Some(end) = self.buffer.find("\n\n") {
                let block: String = self.buffer.drain(..end + 2).collect();
                if let Some(frame) = frame_of(&block) {
                    return Some(frame);
                }
                continue;
            }
            let chunk = tokio::time::timeout(Duration::from_secs(10), self.response.chunk()).await.expect("an event within 10 s").expect("the stream is readable")?;
            self.buffer.push_str(&String::from_utf8_lossy(&chunk));
        }
    }

    async fn until(&mut self, mut found: impl FnMut(&Value) -> bool) -> Vec<Frame> {
        let mut seen = Vec::new();
        while let Some(frame) = self.next().await {
            let done = found(&frame.event);
            seen.push(frame);
            if done {
                return seen;
            }
        }
        panic!("the stream ended before the event was seen: {:?}", seen.iter().map(|frame| &frame.event["type"]).collect::<Vec<_>>());
    }
}

fn frame_of(block: &str) -> Option<Frame> {
    let mut id = None;
    let mut data = String::new();
    for line in block.lines() {
        if let Some(value) = line.strip_prefix("id:") {
            id = value.trim().parse().ok();
        } else if let Some(value) = line.strip_prefix("data:") {
            data.push_str(value.trim_start());
        }
    }
    Some(Frame { id: id?, event: serde_json::from_str(&data).ok()? })
}

fn is_turn_finished(event: &Value) -> bool {
    event["type"] == "turnFinished"
}

fn types(frames: &[Frame]) -> Vec<String> {
    frames.iter().map(|frame| frame.event["type"].as_str().unwrap_or_default().to_string()).collect()
}

async fn chat_app(stubs: &tempfile::TempDir) -> App {
    let received = stubs.path().join("received.txt");
    let stub = stubs.path().join("stub-claude");
    std::fs::write(
        &stub,
        format!("#!/bin/sh\nexport FAKE_CLAUDE_FIXTURE='{PLAIN_REPLY}' FAKE_CLAUDE_RECEIVED='{}'\nexec '{FAKE_CLAUDE}' \"$@\"\n", received.display()),
    )
    .unwrap();
    std::fs::set_permissions(&stub, std::fs::Permissions::from_mode(0o755)).unwrap();
    app_with(
        &[("rekall.claude.cli-path", stub.to_str().unwrap()), ("rekall.terminal.sweep-minutes", "60")],
        StartOptions { no_restart: true, ..StartOptions::default() },
    )
    .await
}

async fn open_chat(app: &App, folder: &std::path::Path) -> String {
    let company = app.a_company("Acme").await;
    let project = app.a_project_in(&company, "alpha", folder).await;
    let task = app.a_task(&project, "one").await;
    let step = app.a_step(&task, "First", None).await;
    let opened = app.post(&format!("/api/tasks/{task}/terminals"), json!({ "stepId": step, "kind": "CHAT", "skipPermissions": true })).await;
    assert_eq!(opened.status, 201);
    assert_eq!(opened.str("kind"), "CHAT");
    opened.str("id")
}

#[tokio::test]
async fn a_chat_session_streams_its_turns_takes_a_message_and_resumes_from_the_last_event_seen() {
    let stubs = tempfile::tempdir().unwrap();
    let app = chat_app(&stubs).await;
    let folder = tempfile::tempdir().unwrap();
    let id = open_chat(&app, folder.path()).await;
    assert_eq!(app.get("/api/terminals").await.list().len(), 1);

    let mut events = EventStream::open(&app, &id, None).await;
    let first_turn = events.until(is_turn_finished).await;
    assert_eq!(first_turn[0].event["type"], "userMessage");
    assert_eq!(first_turn[0].event["text"], "/rk project:alpha task:one");
    assert!(first_turn.iter().any(|frame| frame.event["type"] == "assistantText" && frame.event["text"] == "pong"));
    assert_eq!(first_turn.iter().map(|frame| frame.id).collect::<Vec<_>>(), (1..=first_turn.len() as u64).collect::<Vec<_>>());

    let sent = app.post(&format!("/api/chat/{id}/messages"), json!({ "text": "  hello  " })).await;
    assert_eq!(sent.status, 204);
    let second_turn = events.until(is_turn_finished).await;
    assert_eq!(second_turn[0].event["type"], "userMessage");
    assert_eq!(second_turn[0].event["text"], "hello");
    assert!(second_turn.iter().any(|frame| frame.event["type"] == "assistantText" && frame.event["text"] == "ping"));

    let last_seen = first_turn.last().unwrap().id;
    let mut resumed = EventStream::open(&app, &id, Some(last_seen)).await;
    let newer = resumed.until(is_turn_finished).await;
    assert_eq!(newer.first().unwrap().id, last_seen + 1);
    assert_eq!(types(&newer), types(&second_turn));

    assert_eq!(app.delete(&format!("/api/terminals/{id}")).await.status, 204);
    app.stop().await;
}

#[tokio::test]
async fn closing_a_chat_session_ends_its_stream_with_the_end_event() {
    let stubs = tempfile::tempdir().unwrap();
    let app = chat_app(&stubs).await;
    let folder = tempfile::tempdir().unwrap();
    let id = open_chat(&app, folder.path()).await;
    let mut events = EventStream::open(&app, &id, None).await;
    events.until(is_turn_finished).await;

    app.delete(&format!("/api/terminals/{id}")).await;

    let ending = events.until(|event| event["type"] == "ended").await;
    assert_eq!(ending.last().unwrap().event["detail"], "Closed from the console.");
    assert!(events.next().await.is_none());
    assert_eq!(app.get(&format!("/api/chat/{id}/events")).await.status, 404);
    app.stop().await;
}

#[tokio::test]
async fn what_is_not_a_live_chat_session_or_a_valid_request_is_refused() {
    let stubs = tempfile::tempdir().unwrap();
    let app = chat_app(&stubs).await;
    let folder = tempfile::tempdir().unwrap();
    let id = open_chat(&app, folder.path()).await;
    let unknown = rekall_kernel::Id::random();

    assert_eq!(app.post(&format!("/api/chat/{id}/messages"), json!({ "text": "   " })).await.status, 400);
    assert_eq!(app.post(&format!("/api/chat/{id}/messages"), json!({ "text": "a".repeat(20_001) })).await.status, 400);
    assert_eq!(app.post(&format!("/api/chat/{unknown}/messages"), json!({ "text": "hi" })).await.status, 404);
    assert_eq!(app.post_empty(&format!("/api/chat/{unknown}/interrupt")).await.status, 404);
    assert_eq!(app.post(&format!("/api/chat/{id}/permissions/not-a-uuid"), json!({})).await.status, 400);
    assert_eq!(app.post(&format!("/api/chat/{id}/permissions/not-a-uuid"), json!({ "allow": true })).await.status, 404);
    assert_eq!(app.post("/api/chat/not-a-uuid/messages", json!({ "text": "hi" })).await.status, 400);

    let foreign = app.client.get(app.url(&format!("/api/chat/{id}/events"))).header("Origin", "https://evil.example").send().await.unwrap();
    assert_eq!(foreign.status(), 403);

    assert_eq!(app.delete(&format!("/api/terminals/{id}")).await.status, 204);
    assert_eq!(app.post(&format!("/api/chat/{id}/messages"), json!({ "text": "hi" })).await.status, 404);
    app.stop().await;
}
