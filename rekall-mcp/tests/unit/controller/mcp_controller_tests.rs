use std::io::Write;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use rekall_kernel::{InternalFailure, RekallError};
use serde_json::{json, Value};

use super::McpController;
use crate::protocol::{McpTool, ToolError};

struct FakeTool {
    answer: fn() -> Result<String, ToolError>,
}

#[async_trait]
impl McpTool for FakeTool {
    fn name(&self) -> &'static str {
        "rekall_fake"
    }

    fn description(&self) -> &'static str {
        "A tool whose answer the test chooses."
    }

    fn input_schema(&self) -> Value {
        json!({ "type": "object" })
    }

    async fn execute(&self, _: Option<&Value>) -> Result<String, ToolError> {
        (self.answer)()
    }
}

#[derive(Clone, Default)]
struct CapturedLog(Arc<Mutex<Vec<u8>>>);

impl Write for CapturedLog {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().expect("the log buffer is never poisoned").extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl CapturedLog {
    fn lines(&self) -> Vec<String> {
        let bytes = self.0.lock().expect("the log buffer is never poisoned").clone();
        String::from_utf8(bytes).expect("log output is UTF-8").lines().map(str::to_string).collect()
    }
}

async fn call_logged(answer: fn() -> Result<String, ToolError>, arguments: Value) -> Vec<String> {
    let log = CapturedLog::default();
    let writer = log.clone();
    let subscriber = tracing_subscriber::fmt().with_ansi(false).with_writer(move || writer.clone()).finish();
    let _guard = tracing::subscriber::set_default(subscriber);
    let controller = McpController::new(vec![Arc::new(FakeTool { answer })]);
    let params = json!({ "name": "rekall_fake", "arguments": arguments });
    let _ = controller.call_tool(Some(&params)).await;
    log.lines().into_iter().filter(|line| line.contains("MCP tool call")).collect()
}

#[tokio::test]
async fn a_successful_call_logs_one_event_with_the_tool_the_anchors_and_the_outcome() {
    let events = call_logged(|| Ok("# Context".into()), json!({ "anchors": "project:vega task:report" })).await;

    assert_eq!(events.len(), 1, "{events:?}");
    assert!(events[0].contains("tool=rekall_fake"), "{}", events[0]);
    assert!(events[0].contains("anchors=\"project:vega task:report\""), "{}", events[0]);
    assert!(events[0].contains("duration_ms="), "{}", events[0]);
    assert!(events[0].contains("outcome=\"ok\""), "{}", events[0]);
}

#[tokio::test]
async fn a_refused_call_logs_a_tool_failure() {
    let events = call_logged(|| Err(ToolError::Failure("not allowed".into())), json!({ "anchors": "task:report" })).await;

    assert_eq!(events.len(), 1, "{events:?}");
    assert!(events[0].contains("outcome=\"tool_failure\""), "{}", events[0]);
}

#[tokio::test]
async fn a_call_that_breaks_inside_logs_an_internal_failure() {
    let events = call_logged(|| Err(ToolError::Other(RekallError::internal(InternalFailure::Database, "boom"))), json!({})).await;

    assert_eq!(events.len(), 1, "{events:?}");
    assert!(events[0].contains("outcome=\"internal_failure\""), "{}", events[0]);
    assert!(!events[0].contains("anchors="), "{}", events[0]);
}

#[tokio::test]
async fn argument_bodies_and_results_never_reach_the_log() {
    let events = call_logged(
        || Ok("the result text".into()),
        json!({ "anchors": "task:report", "body": "a private wrapup" }),
    )
    .await;

    assert!(!events[0].contains("a private wrapup"), "{}", events[0]);
    assert!(!events[0].contains("the result text"), "{}", events[0]);
}
