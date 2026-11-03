use async_trait::async_trait;
use rekall_http::guard::RunningStepResponse;

use super::*;
use crate::guard_command::context_source::ContextUnavailable;

const CREDENTIAL_FILE: &str = "/home/dev/.rekall/secrets.json";

struct FakeRekall(Result<GuardContextResponse, String>);

#[async_trait]
impl GuardContextSource for FakeRekall {
    async fn context_of(&self, _folder: &str) -> Result<GuardContextResponse, ContextUnavailable> {
        self.0.clone().map_err(ContextUnavailable)
    }
}

fn context(auto_commit: bool, running_step_number: Option<u32>) -> FakeRekall {
    FakeRekall(Ok(GuardContextResponse {
        project_anchor: Some("project:vega".into()),
        auto_commit,
        running_step: running_step_number.map(|number| RunningStepResponse { number, task_anchor: "project:vega task:report".into() }),
        protected_credential_references: vec![CREDENTIAL_FILE.into()],
    }))
}

fn shell_input(command: &str) -> String {
    format!(r#"{{"cwd":"/work/vega","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{{"command":"{command}"}}}}"#)
}

#[tokio::test]
async fn a_commit_on_an_auto_commit_project_exits_2_with_the_reason() {
    let outcome = run(GuardMode::BeforeTool, &shell_input("git commit -m x"), &context(true, None)).await;

    assert_eq!(outcome.exit_code, 2);
    assert!(outcome.reason.unwrap().contains("git commit"));
}

#[tokio::test]
async fn the_same_commit_exits_0_without_auto_commit() {
    assert_eq!(run(GuardMode::BeforeTool, &shell_input("git commit -m x"), &context(false, None)).await, GuardOutcome::allow());
}

#[tokio::test]
async fn reading_a_credential_file_with_a_file_tool_exits_2() {
    let input = format!(r#"{{"cwd":"/work/vega","tool_name":"Read","tool_input":{{"file_path":"{CREDENTIAL_FILE}"}}}}"#);

    assert_eq!(run(GuardMode::BeforeTool, &input, &context(false, None)).await.exit_code, 2);
}

#[tokio::test]
async fn a_rekall_that_does_not_answer_lets_the_action_through() {
    let unreachable = FakeRekall(Err("connection refused".into()));

    assert_eq!(run(GuardMode::BeforeTool, &shell_input("git commit -m x"), &unreachable).await, GuardOutcome::allow());
}

#[tokio::test]
async fn an_input_that_is_not_json_lets_the_action_through() {
    assert_eq!(run(GuardMode::BeforeTool, "not json", &context(true, None)).await, GuardOutcome::allow());
}

#[tokio::test]
async fn stopping_with_a_running_step_exits_2_and_names_the_step() {
    let outcome = run(GuardMode::BeforeStop, r#"{"cwd":"/work/vega","hook_event_name":"Stop"}"#, &context(false, Some(3))).await;

    assert_eq!(outcome.exit_code, 2);
    assert!(outcome.reason.unwrap().contains("step:3 done"));
}

#[tokio::test]
async fn a_second_stop_after_the_block_is_let_through_so_the_session_can_explain() {
    let input = r#"{"cwd":"/work/vega","hook_event_name":"Stop","stop_hook_active":true}"#;

    assert_eq!(run(GuardMode::BeforeStop, input, &context(false, Some(3))).await, GuardOutcome::allow());
}
