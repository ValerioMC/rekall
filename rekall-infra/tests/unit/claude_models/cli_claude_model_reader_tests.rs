#![cfg(unix)]

use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::Arc;

use serde_json::Value;

use super::*;
use crate::cli::EnvironmentSource;

const FAKE_CLI: &str = include_str!("../../fixtures/fake_claude_models.sh");

struct TestShell;

#[async_trait]
impl EnvironmentSource for TestShell {
    async fn current(&self) -> HashMap<String, String> {
        HashMap::from([("PATH".into(), "/usr/bin:/bin".into())])
    }
}

fn reader_at(binary: PathBuf, home: &Path) -> CliClaudeModelReader {
    CliClaudeModelReader::new(ClaudeCli::new(Some(&binary.to_string_lossy()), home.to_owned(), Arc::new(TestShell)))
}

fn reader_running(home: &Path, script: &str) -> CliClaudeModelReader {
    let binary = home.join("claude");
    fs::write(&binary, script).unwrap();
    fs::set_permissions(&binary, fs::Permissions::from_mode(0o755)).unwrap();
    reader_at(binary, home)
}

fn offer(home: &Path, models: &str) {
    fs::write(home.join("models.json"), models).unwrap();
}

async fn refusal_of(reader: CliClaudeModelReader) -> String {
    match reader.models().await {
        Err(RekallError::Conflict(message)) => message,
        other => panic!("expected the catalog to be unavailable, got {other:?}"),
    }
}

#[tokio::test]
async fn each_read_returns_what_the_cli_offers_at_that_moment() {
    let home = tempfile::tempdir().unwrap();
    let reader = reader_running(home.path(), FAKE_CLI);

    offer(home.path(), r#"[{"value":"sonnet","displayName":"Sonnet"}]"#);
    let before = reader.models().await.unwrap();
    offer(home.path(), r#"[{"value":"sonnet","displayName":"Sonnet"},{"value":"claude-example-7[1m]","displayName":"Example 7","resolvedModel":"claude-example-7","supportsEffort":true,"supportedEffortLevels":["high"],"supportsFastMode":true}]"#);
    let after = reader.models().await.unwrap();

    assert_eq!(before.len(), 1);
    assert_eq!(
        after[1],
        ClaudeModel {
            value: "claude-example-7[1m]".into(),
            display_name: "Example 7".into(),
            resolved_model: Some("claude-example-7".into()),
            supports_effort: Some(true),
            supported_effort_levels: Some(vec!["high".into()]),
        }
    );
}

#[tokio::test]
async fn omitted_effort_metadata_stays_apart_from_a_declared_lack_of_it() {
    let home = tempfile::tempdir().unwrap();
    let reader = reader_running(home.path(), FAKE_CLI);
    offer(home.path(), r#"[{"value":"older","displayName":"Older"},{"value":"plain","displayName":"Plain","supportsEffort":false}]"#);

    let models = reader.models().await.unwrap();

    assert_eq!((models[0].supports_effort, &models[0].supported_effort_levels, &models[0].resolved_model), (None, &None, &None));
    assert_eq!(models[1].supports_effort, Some(false));
}

#[tokio::test]
async fn the_cli_is_sent_only_the_initialize_message_from_a_folder_removed_afterwards() {
    let home = tempfile::tempdir().unwrap();
    let reader = reader_running(home.path(), FAKE_CLI);
    offer(home.path(), "[]");

    reader.models().await.unwrap();

    let arguments = fs::read_to_string(home.path().join("arguments")).unwrap();
    assert_eq!(arguments.lines().collect::<Vec<_>>(), CATALOG_ARGUMENTS);
    let received = fs::read_to_string(home.path().join("received")).unwrap();
    let messages: Vec<Value> = received.lines().map(|line| serde_json::from_str(line).unwrap()).collect();
    assert_eq!(messages, [json!({ "type": "control_request", "request_id": REQUEST_ID, "request": { "subtype": "initialize" } })]);
    let folder = fs::read_to_string(home.path().join("working-directory")).unwrap();
    assert!(!Path::new(folder.trim()).exists(), "{folder} was left behind");
}

#[tokio::test]
async fn a_missing_cli_leaves_the_catalog_unavailable() {
    let home = tempfile::tempdir().unwrap();

    let message = refusal_of(reader_at(home.path().join("missing"), home.path())).await;

    assert!(message.contains("isn't on this machine"), "{message}");
}

#[tokio::test]
async fn a_cli_that_exits_without_answering_leaves_the_catalog_unavailable() {
    let home = tempfile::tempdir().unwrap();

    let message = refusal_of(reader_running(home.path(), "#!/bin/sh\necho \"error: unknown option '--safe-mode'\" >&2\nexit 1\n")).await;

    assert!(message.starts_with("Claude Code models are unavailable"), "{message}");
}

#[tokio::test]
async fn a_cli_that_refuses_the_request_leaves_the_catalog_unavailable_with_its_reason() {
    let home = tempfile::tempdir().unwrap();
    let script = r#"#!/bin/sh
read -r line
printf '{"type":"control_response","response":{"subtype":"error","request_id":"rekall-models","error":"Unsupported control request"}}\n'
"#;

    let message = refusal_of(reader_running(home.path(), script)).await;

    assert!(message.contains("Unsupported control request"), "{message}");
}

#[tokio::test]
async fn an_answer_without_a_model_list_leaves_the_catalog_unavailable() {
    let home = tempfile::tempdir().unwrap();
    let script = r#"#!/bin/sh
read -r line
printf '{"type":"control_response","response":{"subtype":"success","request_id":"rekall-models","response":{"commands":[]}}}\n'
"#;

    let message = refusal_of(reader_running(home.path(), script)).await;

    assert!(message.contains("models"), "{message}");
}

#[tokio::test]
async fn a_silent_cli_is_killed_when_its_time_runs_out() {
    let home = tempfile::tempdir().unwrap();
    let script = "#!/bin/sh\necho $$ > \"$(dirname \"$0\")/pid\"\nexec sleep 30\n";
    let reader = reader_running(home.path(), script).with_answer_timeout(Duration::from_secs(2));

    let message = refusal_of(reader).await;

    assert!(message.contains("did not answer in time"), "{message}");
    let pid: libc::pid_t = fs::read_to_string(home.path().join("pid")).unwrap().trim().parse().unwrap();
    // SAFETY: signal 0 only asks whether the process exists.
    assert_eq!(unsafe { libc::kill(pid, 0) }, -1, "process {pid} is still running");
}
