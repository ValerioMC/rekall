use crate::terminal::OpenTerminalRequest;
use rekall_diagram::DiagramKind;

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
fn a_plan_flow_terminal_asks_the_session_for_a_flow_and_follows_no_step() {
    assert_eq!(TerminalMode::PlanFlow.first_line(ANCHORS), "/rk project:vega task:report-builder plan flow");
    assert!(!TerminalMode::PlanFlow.follows_steps());
    let request: OpenTerminalRequest = serde_json::from_str(r#"{"mode":"PLAN_FLOW"}"#).unwrap();
    assert_eq!(request.mode().unwrap(), TerminalMode::PlanFlow);
}

#[test]
fn a_generate_terminal_carries_the_request_as_one_quoted_line() {
    let mode = TerminalMode::Generate { request: "How does an order\nget \"paid\"?  ".into(), kinds: vec![] };
    assert_eq!(mode.first_line(ANCHORS), "/rk project:vega task:report-builder generate \"How does an order get 'paid'?\"");
    assert!(!mode.follows_steps());
}

#[test]
fn a_generate_terminal_names_the_diagram_types_asked_for() {
    let mode = TerminalMode::Generate { request: "checkout".into(), kinds: vec![DiagramKind::Sequence, DiagramKind::Er] };
    assert_eq!(mode.first_line("project:vega"), "/rk project:vega generate kinds:sequence,er \"checkout\"");
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
    assert_eq!(asked.mode().unwrap(), TerminalMode::Generate { request: "the order flow".into(), kinds: vec![] });
}

#[test]
fn generate_kinds_are_read_in_order_once_each_and_an_unknown_one_is_refused() {
    let asked: OpenTerminalRequest =
        serde_json::from_str(r#"{"mode":"GENERATE","request":"x","kinds":["state","sequence","state"]}"#).unwrap();
    assert_eq!(asked.mode().unwrap(), TerminalMode::Generate { request: "x".into(), kinds: vec![DiagramKind::State, DiagramKind::Sequence] });
    let unknown: OpenTerminalRequest = serde_json::from_str(r#"{"mode":"GENERATE","request":"x","kinds":["uml"]}"#).unwrap();
    assert!(unknown.mode().is_err());
}

#[test]
fn an_explain_terminal_asks_for_a_walkthrough_of_the_request() {
    let mode = TerminalMode::Explain { request: "How an order\nis \"placed\"".into() };
    assert_eq!(mode.first_line("project:vega"), "/rk project:vega explain \"How an order is 'placed'\"");
    assert!(mode.draws_a_diagram());
    assert!(!TerminalMode::Plan.draws_a_diagram());
}

#[test]
fn an_explain_request_needs_something_to_explain() {
    let blank: OpenTerminalRequest = serde_json::from_str(r#"{"mode":"EXPLAIN"}"#).unwrap();
    assert!(blank.mode().is_err());
    let asked: OpenTerminalRequest = serde_json::from_str(r#"{"mode":"EXPLAIN","request":"checkout"}"#).unwrap();
    assert_eq!(asked.mode().unwrap(), TerminalMode::Explain { request: "checkout".into() });
}
