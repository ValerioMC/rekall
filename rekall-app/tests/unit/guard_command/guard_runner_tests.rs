use async_trait::async_trait;
use rekall_http::guard::RunningStepResponse;

use super::*;
use crate::guard_command::context_source::ContextUnavailable;

const CREDENTIAL_FILE: &str = "/home/dev/.rekall/secrets.json";

struct FakeRekall(Result<GuardContextResponse, String>);

#[async_trait]
impl GuardContextSource for FakeRekall {
    async fn context_of(&self, _folder: &str, _scope: Option<GuardScope>) -> Result<GuardContextResponse, ContextUnavailable> {
        self.0.clone().map_err(ContextUnavailable)
    }
}

fn context(auto_commit: bool, running_step_number: Option<u32>) -> FakeRekall {
    FakeRekall(Ok(GuardContextResponse {
        project_anchor: Some("project:vega".into()),
        auto_commit,
        running_step: running_step_number.map(|number| RunningStepResponse { number, task_anchor: "task:VE-12".into() }),
        protected_credential_references: vec![CREDENTIAL_FILE.into()],
    }))
}

fn shell_input(command: &str) -> String {
    format!(r#"{{"cwd":"/work/vega","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{{"command":"{command}"}}}}"#)
}

async fn run_claude(mode: GuardMode, input: &str, source: &dyn GuardContextSource) -> GuardOutcome {
    run(SessionProvider::Claude, mode, input, None, source).await
}

#[tokio::test]
async fn a_commit_on_an_auto_commit_project_exits_2_with_the_reason() {
    let outcome = run_claude(GuardMode::BeforeTool, &shell_input("git commit -m x"), &context(true, None)).await;

    assert_eq!(outcome.exit_code, 2);
    assert!(outcome.stderr.unwrap().contains("git commit"));
}

#[tokio::test]
async fn the_same_commit_exits_0_without_auto_commit() {
    assert_eq!(run_claude(GuardMode::BeforeTool, &shell_input("git commit -m x"), &context(false, None)).await, GuardOutcome::allow());
}

#[tokio::test]
async fn reading_a_credential_file_with_a_file_tool_exits_2() {
    let input = format!(r#"{{"cwd":"/work/vega","tool_name":"Read","tool_input":{{"file_path":"{CREDENTIAL_FILE}"}}}}"#);

    assert_eq!(run_claude(GuardMode::BeforeTool, &input, &context(false, None)).await.exit_code, 2);
}

#[tokio::test]
async fn a_rekall_that_does_not_answer_lets_the_action_through() {
    let unreachable = FakeRekall(Err("connection refused".into()));

    assert_eq!(run_claude(GuardMode::BeforeTool, &shell_input("git commit -m x"), &unreachable).await, GuardOutcome::allow());
}

#[tokio::test]
async fn an_input_that_is_not_json_lets_the_action_through() {
    assert_eq!(run_claude(GuardMode::BeforeTool, "not json", &context(true, None)).await, GuardOutcome::allow());
}

#[tokio::test]
async fn stopping_with_a_running_step_exits_2_and_names_the_step() {
    let outcome = run_claude(GuardMode::BeforeStop, r#"{"cwd":"/work/vega","hook_event_name":"Stop"}"#, &context(false, Some(3))).await;

    assert_eq!(outcome.exit_code, 2);
    assert!(outcome.stderr.unwrap().contains("step:3 done"));
}

#[tokio::test]
async fn a_second_stop_after_the_block_is_let_through_so_the_session_can_explain() {
    let input = r#"{"cwd":"/work/vega","hook_event_name":"Stop","stop_hook_active":true}"#;

    assert_eq!(run_claude(GuardMode::BeforeStop, input, &context(false, Some(3))).await, GuardOutcome::allow());
}

#[tokio::test]
async fn codex_shell_pre_tool_use_returns_a_deny_without_bypassing_permissions() {
    let command = r#"{"session_id":"thread","cwd":"/work/vega","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"git push","workdir":"/work/vega"}}"#;
    let outcome = run(SessionProvider::Codex, GuardMode::BeforeTool, command, None, &context(true, None)).await;

    assert_eq!(outcome.exit_code, 0);
    assert_eq!(outcome.stderr, None);
    let output: serde_json::Value = serde_json::from_str(&outcome.stdout.unwrap()).unwrap();
    assert_eq!(output["hookSpecificOutput"]["hookEventName"], "PreToolUse");
    assert_eq!(output["hookSpecificOutput"]["permissionDecision"], "deny");
    assert!(output["hookSpecificOutput"]["permissionDecisionReason"].as_str().unwrap().contains("git push"));
    assert_eq!(run(SessionProvider::Codex, GuardMode::BeforeTool, command, None, &context(false, None)).await, GuardOutcome::allow());
}

#[tokio::test]
async fn codex_checks_credential_references_inside_non_shell_tool_arguments() {
    let input = format!(r#"{{"cwd":"/work/vega","hook_event_name":"PreToolUse","tool_name":"mcp__files__read","tool_input":{{"request":{{"paths":["{CREDENTIAL_FILE}"]}}}}}}"#);
    let outcome = run(SessionProvider::Codex, GuardMode::BeforeTool, &input, None, &context(false, None)).await;

    assert!(outcome.stdout.unwrap().contains("Rekall's credentials"));
}

#[tokio::test]
async fn editing_a_file_checks_its_path_without_treating_its_contents_as_a_read() {
    let claude = serde_json::json!({
        "cwd": "/work/vega", "tool_name": "Write",
        "tool_input": { "file_path": "/work/vega/README.md", "content": CREDENTIAL_FILE }
    });
    assert_eq!(run_claude(GuardMode::BeforeTool, &claude.to_string(), &context(true, None)).await, GuardOutcome::allow());

    for (path, blocked) in [("/work/vega/README.md", false), (CREDENTIAL_FILE, true)] {
        let codex = serde_json::json!({
            "cwd": "/work/vega", "hook_event_name": "PreToolUse", "tool_name": "apply_patch",
            "tool_input": { "command": format!("*** Begin Patch\n*** Add File: {path}\n+{CREDENTIAL_FILE}\n*** End Patch") }
        });
        let outcome = run(SessionProvider::Codex, GuardMode::BeforeTool, &codex.to_string(), None, &context(true, None)).await;
        assert_eq!(outcome.stdout.is_some(), blocked);
    }
}

#[tokio::test]
async fn codex_stop_blocks_once_with_the_exact_claim_command() {
    let input = |active| format!(r#"{{"session_id":"thread","cwd":"/work/vega","hook_event_name":"Stop","stop_hook_active":{active}}}"#);
    let stop = run(SessionProvider::Codex, GuardMode::BeforeStop, &input(false), None, &context(true, Some(2))).await;

    assert_eq!(stop.exit_code, 0);
    assert_eq!(stop.stderr, None);
    let output: serde_json::Value = serde_json::from_str(&stop.stdout.unwrap()).unwrap();
    assert_eq!(output["decision"], "block");
    assert!(output["reason"].as_str().unwrap().contains("/rk task:VE-12 step:2 done"));
    assert_eq!(run(SessionProvider::Codex, GuardMode::BeforeStop, &input(true), None, &context(true, Some(2))).await, GuardOutcome::allow());
    assert_eq!(run(SessionProvider::Codex, GuardMode::BeforeStop, &input(false), None, &context(true, None)).await, GuardOutcome::allow());
}

#[tokio::test]
async fn codex_allows_an_unavailable_rekall_or_an_invalid_event() {
    let no_server = FakeRekall(Err("connection refused".into()));
    let input = r#"{"cwd":"/work/vega","hook_event_name":"Stop","stop_hook_active":false}"#;

    assert_eq!(run(SessionProvider::Codex, GuardMode::BeforeStop, input, None, &no_server).await, GuardOutcome::allow());
    for malformed in ["not JSON", r#"{"cwd":"/work/vega"}"#, r#"{"cwd":"/work/vega","hook_event_name":"PreToolUse"}"#] {
        assert_eq!(run(SessionProvider::Codex, GuardMode::BeforeStop, malformed, None, &context(true, Some(2))).await, GuardOutcome::allow());
    }
}
