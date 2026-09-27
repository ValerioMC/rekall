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
fn a_generate_terminal_carries_the_request_as_one_quoted_line() {
    let mode = TerminalMode::Generate { request: "How does an order\nget \"paid\"?  ".into() };
    assert_eq!(mode.first_line(ANCHORS), "/rk project:vega task:report-builder generate \"How does an order get 'paid'?\"");
    assert!(!mode.follows_steps());
}

#[test]
fn a_request_without_a_mode_opens_a_work_terminal() {
    let request: OpenTerminalRequest = serde_json::from_str("{}").unwrap();
    assert_eq!(request.mode().unwrap(), TerminalMode::Work);
    let request: OpenTerminalRequest = serde_json::from_str(r#"{"mode":"PLAN"}"#).unwrap();
    assert_eq!(request.mode().unwrap(), TerminalMode::Plan);
}

#[test]
fn a_generate_request_needs_something_to_generate() {
    let blank: OpenTerminalRequest = serde_json::from_str(r#"{"mode":"GENERATE","request":"  "}"#).unwrap();
    assert!(blank.mode().is_err());
    let asked: OpenTerminalRequest = serde_json::from_str(r#"{"mode":"GENERATE","request":"the order flow"}"#).unwrap();
    assert_eq!(asked.mode().unwrap(), TerminalMode::Generate { request: "the order flow".into() });
}
