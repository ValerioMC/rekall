use super::*;

const SLASH: &str = r#"{"type":"user","timestamp":"2026-10-01T10:00:00.000Z","message":{"role":"user","content":"<command-name>/rk</command-name>\n<command-args>ARGS</command-args>"}}"#;

fn slash(args: &str, at: &str) -> String {
    SLASH.replace("ARGS", args).replace("10:00:00", at)
}

fn call(id: &str, at: &str, output: u64) -> String {
    format!(
        r#"{{"type":"assistant","timestamp":"2026-10-01T{at}.000Z","message":{{"id":"{id}","content":[{{"type":"text","text":"ok"}}],"usage":{{"input_tokens":1,"output_tokens":{output},"cache_creation_input_tokens":10,"cache_read_input_tokens":100}}}}}}"#
    )
}

fn tool(id: &str, at: &str, name: &str, input: &str) -> String {
    format!(
        r#"{{"type":"assistant","timestamp":"2026-10-01T{at}.000Z","message":{{"id":"{id}","content":[{{"type":"tool_use","name":"{name}","input":{input}}}],"usage":{{"input_tokens":1,"output_tokens":1,"cache_creation_input_tokens":0,"cache_read_input_tokens":0}}}}}}"#
    )
}

fn output_of(spent: &[UnitUsage], task: &str, step: Option<&str>) -> Option<u64> {
    spent.iter().find(|usage| usage.unit.task_label == task && usage.unit.step.as_deref() == step).map(|usage| usage.tokens.output)
}

#[test]
fn a_call_before_any_rekall_work_is_not_counted() {
    let mut ledger = SessionLedger::new();
    ledger.read_line(&call("m1", "09:00:00", 50));
    assert!(ledger.spent().is_empty());
}

#[test]
fn calls_after_an_rk_line_are_charged_to_its_task_with_all_four_counts() {
    let mut ledger = SessionLedger::new();
    ledger.read_line(&slash("project:vega task:report", "10:00:00"));
    ledger.read_line(&call("m1", "10:00:01", 50));
    ledger.read_line(&call("m2", "10:00:02", 25));

    let spent = ledger.spent();
    assert_eq!(spent.len(), 1);
    assert_eq!(spent[0].unit, WorkUnit::task(Some("vega"), "report"));
    assert_eq!(spent[0].tokens, TokenCount { input: 2, output: 75, cache_creation: 20, cache_read: 200 });
}

#[test]
fn a_message_written_on_several_lines_is_charged_once() {
    let mut ledger = SessionLedger::new();
    ledger.read_line(&slash("task:report", "10:00:00"));
    ledger.read_line(&call("m1", "10:00:01", 40));
    ledger.read_line(&call("m1", "10:00:01", 50));

    assert_eq!(output_of(&ledger.spent(), "report", None), Some(50));
}

#[test]
fn a_running_step_takes_the_calls_until_its_claim_and_the_task_takes_the_rest() {
    let mut ledger = SessionLedger::new();
    ledger.read_line(&slash("project:vega task:report", "10:00:00"));
    ledger.read_line(&call("m1", "10:00:01", 5));
    ledger.read_line(&tool("m2", "10:00:02", "mcp__rekall__rekall_step", r#"{"anchors":"project:vega task:report","step":"2","state":"running"}"#));
    ledger.read_line(&call("m3", "10:00:03", 7));
    ledger.read_line(&slash("project:vega task:report wrapup", "10:00:04"));
    ledger.read_line(&call("m4", "10:00:05", 11));
    ledger.read_line(&tool("m5", "10:00:06", "mcp__rekall__rekall_step", r#"{"anchors":"project:vega task:report","step":2,"state":"claimed"}"#));
    ledger.read_line(&call("m6", "10:00:07", 13));

    let spent = ledger.spent();
    assert_eq!(output_of(&spent, "report", None), Some(5 + 1 + 13));
    assert_eq!(output_of(&spent, "report", Some("2")), Some(7 + 11 + 1));
}

#[test]
fn a_context_call_on_another_task_moves_the_session_to_it() {
    let mut ledger = SessionLedger::new();
    ledger.read_line(&slash("task:report", "10:00:00"));
    ledger.read_line(&tool("m1", "10:00:01", "rekall_context", r#"{"anchors":"project:vega task:export"}"#));
    ledger.read_line(&call("m2", "10:00:02", 9));
    ledger.read_line(&tool("m3", "10:00:03", "mcp__rekall__rekall_context", r#"{"anchors":"note:3f2a9c1e"}"#));
    ledger.read_line(&call("m4", "10:00:04", 4));

    let spent = ledger.spent();
    assert_eq!(output_of(&spent, "report", None), Some(1));
    assert_eq!(output_of(&spent, "export", None), Some(9 + 1 + 4));
}

#[test]
fn a_subagent_call_is_charged_to_what_the_session_was_on_at_its_time() {
    let mut ledger = SessionLedger::new();
    ledger.read_line(&slash("task:report step:1 start", "10:00:00"));
    ledger.read_line(&slash("task:export", "10:05:00"));
    ledger.read_subagent_line(&call("a1", "10:01:00", 30));
    ledger.read_subagent_line(&call("a2", "10:06:00", 20));
    ledger.read_subagent_line(&call("a3", "09:00:00", 99));

    let spent = ledger.spent();
    assert_eq!(output_of(&spent, "report", Some("1")), Some(30));
    assert_eq!(output_of(&spent, "export", None), Some(20));
    assert_eq!(spent.len(), 2);
}

#[test]
fn a_line_that_is_not_json_or_has_an_unexpected_shape_costs_nothing_else() {
    let mut ledger = SessionLedger::new();
    ledger.read_line(&slash("task:report", "10:00:00"));
    ledger.read_line(r#"{"type":"assistant","message":{"id":"m1","usage":{"output_tokens":3}"#);
    ledger.read_line(r#"{"type":"assistant","message":{"id":"m2","content":[{"no_type":true}],"usage":{"output_tokens":3}}}"#);
    ledger.read_line(&tool("m3", "10:00:01", "Bash", r#"{"anchors":["odd"],"step":{"x":1}}"#));

    assert_eq!(output_of(&ledger.spent(), "report", None), Some(3 + 1));
}
