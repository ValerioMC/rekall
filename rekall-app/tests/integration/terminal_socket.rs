//! The terminal socket end to end: a pane gets ready, hears the session's output, types into it,
//! resizes it and hears it end; a handshake from a foreign origin is refused.

mod support;

use std::os::unix::fs::PermissionsExt;
use std::time::Duration;

use base64::Engine;
use futures::{SinkExt, StreamExt};
use rekall_app::StartOptions;
use serde_json::{json, Value};
use support::{app_with, App};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;

/// Prints its arguments, then echoes each line it reads; `exit` ends it with code 3.
const STUB_CLAUDE: &str = r#"#!/bin/sh
echo "ARGS:$*"
while IFS= read -r line; do
  if [ "$line" = "exit" ]; then
    echo "BYE"
    exit 3
  fi
  echo "GOT:$line"
done
"#;

async fn socket_app(stubs: &tempfile::TempDir) -> App {
    let stub = stubs.path().join("stub-claude");
    std::fs::write(&stub, STUB_CLAUDE).unwrap();
    std::fs::set_permissions(&stub, std::fs::Permissions::from_mode(0o755)).unwrap();
    app_with(
        &[("rekall.claude.cli-path", stub.to_str().unwrap()), ("rekall.terminal.sweep-minutes", "60")],
        StartOptions { no_restart: true, ..StartOptions::default() },
    )
    .await
}

fn host_of(app: &App) -> String {
    format!("localhost:{}", app.running.as_ref().unwrap().port)
}

#[tokio::test]
async fn a_pane_gets_ready_then_output_sends_input_and_hears_the_end() {
    let stubs = tempfile::tempdir().unwrap();
    let app = socket_app(&stubs).await;
    let folder = tempfile::tempdir().unwrap();
    let company = app.a_company("Acme").await;
    let project = app.a_project_in(&company, "alpha", folder.path()).await;
    let task = app.a_task(&project, "one").await;
    let step = app.a_step(&task, "First", None).await;

    let opened = app.post(&format!("/api/tasks/{task}/terminals"), json!({ "stepId": step, "model": "sonnet" })).await;
    assert_eq!(opened.status, 201);
    assert_eq!(opened.str("anchors"), "project:alpha task:one");
    assert_eq!(opened.str("stepId"), step);
    assert_eq!(opened.str("model"), "sonnet");
    assert_eq!(opened.get("live"), &json!(true));
    let id = opened.str("id");
    assert_eq!(app.get("/api/terminals").await.list().len(), 1);

    let mut request = format!("ws://{}/api/terminal/{id}/io", host_of(&app)).into_client_request().unwrap();
    request.headers_mut().insert("Origin", "http://localhost:5173".parse().unwrap());
    let (mut socket, _) = tokio_tungstenite::connect_async(request).await.unwrap();

    let ready = next_text(&mut socket).await;
    assert_eq!(ready["type"], "ready");
    assert_eq!(ready["id"], id);
    assert_eq!(ready["workingDir"], folder.path().to_string_lossy().as_ref());

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
    assert_eq!(ended, json!({ "type": "ended", "exitCode": 3, "detail": "claude exited with code 3" }));

    assert_eq!(app.get(&format!("/api/terminals/{id}")).await.status, 404);
    app.stop().await;
}

#[tokio::test]
async fn a_handshake_from_a_foreign_origin_is_refused() {
    let stubs = tempfile::tempdir().unwrap();
    let app = socket_app(&stubs).await;
    let mut request = format!("ws://{}/api/terminal/{}/io", host_of(&app), rekall_kernel::Id::random()).into_client_request().unwrap();
    request.headers_mut().insert("Origin", "https://evil.example".parse().unwrap());

    let refused = tokio_tungstenite::connect_async(request).await.unwrap_err();

    assert!(refused.to_string().contains("403"), "{refused}");
    app.stop().await;
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
