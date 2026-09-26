use crate::terminal::OpenTerminalRequest;

use super::*;

const ANCHORS: &str = "project:vega task:report-builder";

#[test]
fn a_work_terminal_loads_the_task() {
    assert_eq!(TerminalMode::Work.first_line(ANCHORS), "/rk project:vega task:report-builder");
}

#[test]
fn a_plan_terminal_asks_the_session_to_plan_the_task() {
    assert_eq!(TerminalMode::Plan.first_line(ANCHORS), "/rk project:vega task:report-builder plan");
}

#[test]
fn a_request_without_a_mode_opens_a_work_terminal() {
    let request: OpenTerminalRequest = serde_json::from_str("{}").unwrap();
    assert_eq!(request.mode_or_default(), TerminalMode::Work);
    let request: OpenTerminalRequest = serde_json::from_str(r#"{"mode":"PLAN"}"#).unwrap();
    assert_eq!(request.mode_or_default(), TerminalMode::Plan);
}
