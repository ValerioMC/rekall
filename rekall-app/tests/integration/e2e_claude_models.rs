//! The model catalog of the installed Claude Code over HTTP, against a CLI of the test's own.

#![cfg(unix)]

mod support;

use std::os::unix::fs::PermissionsExt;

use rekall_app::StartOptions;
use serde_json::json;
use support::{app_with, App};
use tempfile::TempDir;

/// An application whose `claude` is `script`, kept in the folder returned with it.
async fn app_running(script: &str) -> (App, TempDir) {
    let stubs = tempfile::tempdir().unwrap();
    let stub = stubs.path().join("claude");
    std::fs::write(&stub, script).unwrap();
    std::fs::set_permissions(&stub, std::fs::Permissions::from_mode(0o755)).unwrap();
    let app = app_with(&[("rekall.claude.cli-path", stub.to_str().unwrap())], StartOptions { no_restart: true, ..StartOptions::default() }).await;
    (app, stubs)
}

#[tokio::test]
async fn the_catalog_is_what_the_cli_answers_now() {
    let (app, stubs) = app_running(include_str!("../../../rekall-infra/tests/fixtures/fake_claude_models.sh")).await;
    let models = stubs.path().join("models.json");
    std::fs::write(&models, "[]").unwrap();

    assert_eq!(app.get("/api/claude/models").await.body, json!([]));

    std::fs::write(
        &models,
        r#"[{"value":"claude-example-7[1m]","displayName":"Example 7","supportsEffort":true,"supportedEffortLevels":["high"]}]"#,
    )
    .unwrap();
    let catalog = app.get("/api/claude/models").await;

    assert_eq!(catalog.status, 200, "{}", catalog.text);
    assert_eq!(
        catalog.body,
        json!([{
            "value": "claude-example-7[1m]",
            "displayName": "Example 7",
            "resolvedModel": null,
            "supportsEffort": true,
            "supportedEffortLevels": ["high"],
        }])
    );
    app.stop().await;
}

#[tokio::test]
async fn an_unsupported_cli_is_a_conflict_the_console_can_retry() {
    let (app, _stubs) = app_running("#!/bin/sh\nexit 1\n").await;

    let refusal = app.get("/api/claude/models").await;

    assert_eq!(refusal.status, 409, "{}", refusal.text);
    assert_contains!(refusal.detail(), "Claude Code models are unavailable");
    app.stop().await;
}
