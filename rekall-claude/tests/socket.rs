//! The terminal pane's wire contract over a real listener: REST to open, the WebSocket to talk.

mod support;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use base64::Engine;
use futures::{SinkExt, StreamExt};
use rekall_api::{ApiState, StepEventStream};
use rekall_claude::queue::{RunQueueRunner, RunQueueService};
use rekall_claude::usage::{ClaudeUsageView, UsageReader};
use rekall_claude::ClaudeState;
use rekall_model::TaskStepState;
use serde_json::Value;
use support::{world, World};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;

struct NoUsage(Mutex<()>);

#[async_trait::async_trait]
impl UsageReader for NoUsage {
    async fn current(&self) -> ClaudeUsageView {
        let _held = self.0.lock().unwrap();
        ClaudeUsageView::unauthenticated()
    }

    async fn refresh(&self) -> ClaudeUsageView {
        ClaudeUsageView::unauthenticated()
    }
}

async fn serve(world: &World) -> String {
    let terminals = world.terminals(8);
    let usage: Arc<dyn UsageReader> = Arc::new(NoUsage(Mutex::new(())));
    let queue = RunQueueService::new(world.services.ctx.clone());
    let runner = RunQueueRunner::start(
        queue.clone(),
        terminals.clone(),
        usage.clone(),
        world.services.task_work.clone(),
        world.services.steps.clone(),
        &world.events,
        rekall_service::system_clock(),
        Duration::from_secs(60),
        Duration::from_secs(1),
    );
    let api = ApiState::new(world.services.clone(), Arc::new(StepEventStream::new(world.events.clone())));
    let state = ClaudeState { api, terminals, usage, queue, runner };
    let app = rekall_claude::router(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("127.0.0.1:{}", address.port())
}

#[tokio::test]
async fn a_pane_gets_ready_then_output_sends_input_and_hears_the_end() {
    let world = world().await;
    let project = world.project("alpha", Some(world.folder.path())).await;
    let task = world.task(&project, "one").await;
    let step = world.step(&task, "First", 0, TaskStepState::Open).await;
    let host = serve(&world).await;

    let client = reqwest::Client::new();
    let opened = client
        .post(format!("http://{host}/api/tasks/{}/terminals", task.id))
        .json(&serde_json::json!({ "stepId": step.id, "model": "sonnet" }))
        .send()
        .await
        .unwrap();
    assert_eq!(opened.status(), 201);
    let opened: Value = opened.json().await.unwrap();
    assert_eq!(opened["anchors"], "project:alpha task:one");
    assert_eq!(opened["stepId"], step.id.to_string());
    assert_eq!(opened["model"], "sonnet");
    assert_eq!(opened["live"], true);
    let id = opened["id"].as_str().unwrap().to_string();

    let listed: Value = client.get(format!("http://{host}/api/terminals")).send().await.unwrap().json().await.unwrap();
    assert_eq!(listed.as_array().unwrap().len(), 1);

    let mut request = format!("ws://{host}/api/terminal/{id}/io").into_client_request().unwrap();
    request.headers_mut().insert("Origin", "http://localhost:5173".parse().unwrap());
    let (mut socket, _) = tokio_tungstenite::connect_async(request).await.unwrap();

    let ready = next_text(&mut socket).await;
    assert_eq!(ready["type"], "ready");
    assert_eq!(ready["id"], id);
    assert_eq!(ready["workingDir"], world.folder.path().to_string_lossy().as_ref());

    let mut seen = String::new();
    read_until(&mut socket, &mut seen, "ARGS:--model sonnet /rk project:alpha task:one").await;

    socket.send(Message::Binary(b"typed\n".to_vec().into())).await.unwrap();
    read_until(&mut socket, &mut seen, "GOT:typed").await;

    socket.send(Message::Text(r#"{"resize":[120,40]}"#.into())).await.unwrap();
    let encoded = base64::engine::general_purpose::STANDARD.encode("exit\n");
    socket.send(Message::Text(format!(r#"{{"in":"{encoded}"}}"#).into())).await.unwrap();

    let ended = loop {
        match tokio::time::timeout(Duration::from_secs(5), socket.next()).await.unwrap().unwrap().unwrap() {
            Message::Text(text) => break serde_json::from_str::<Value>(&text).unwrap(),
            Message::Binary(_) => continue,
            other => panic!("unexpected frame {other:?}"),
        }
    };
    assert_eq!(ended, serde_json::json!({ "type": "ended", "exitCode": 3, "detail": "claude exited with code 3" }));

    let gone = client.get(format!("http://{host}/api/terminals/{id}")).send().await.unwrap();
    assert_eq!(gone.status(), 404);
}

#[tokio::test]
async fn a_handshake_from_a_foreign_origin_is_refused() {
    let world = world().await;
    let host = serve(&world).await;
    let mut request = format!("ws://{host}/api/terminal/{}/io", uuid_like()).into_client_request().unwrap();
    request.headers_mut().insert("Origin", "https://evil.example".parse().unwrap());
    let refused = tokio_tungstenite::connect_async(request).await.unwrap_err();
    assert!(refused.to_string().contains("403"), "{refused}");
}

fn uuid_like() -> String {
    rekall_common::Id::random().to_string()
}

async fn next_text<S>(socket: &mut S) -> Value
where
    S: StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin,
{
    match tokio::time::timeout(Duration::from_secs(5), socket.next()).await.unwrap().unwrap().unwrap() {
        Message::Text(text) => serde_json::from_str(&text).unwrap(),
        other => panic!("expected a text frame, got {other:?}"),
    }
}

async fn read_until<S>(socket: &mut S, seen: &mut String, wanted: &str)
where
    S: StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin,
{
    while !seen.contains(wanted) {
        match tokio::time::timeout(Duration::from_secs(5), socket.next()).await.unwrap().unwrap().unwrap() {
            Message::Binary(bytes) => seen.push_str(&String::from_utf8_lossy(&bytes)),
            other => panic!("expected output, got {other:?}"),
        }
    }
}
