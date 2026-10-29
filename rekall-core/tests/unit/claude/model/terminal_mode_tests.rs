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
fn an_explain_terminal_asks_for_a_walkthrough_of_the_request() {
    let mode = TerminalMode::Explain { request: "How an order\nis \"placed\"".into() };
    assert_eq!(mode.first_line("project:vega"), "/rk project:vega explain \"How an order is 'placed'\"");
    assert!(mode.works_on_a_project());
    assert!(!TerminalMode::Plan.works_on_a_project());
}

#[test]
fn an_inventory_terminal_asks_the_session_to_record_the_projects_dependencies() {
    let mode = TerminalMode::Inventory;
    assert_eq!(mode.first_line("project:vega"), "/rk project:vega inventory");
    assert!(mode.works_on_a_project() && !mode.follows_steps());
}

#[test]
fn an_assess_terminal_names_the_advisory_by_its_ghsa_id() {
    let mode = TerminalMode::Assess { advisory: "GHSA-jfh8-c2jp-5v3q".into() };
    assert_eq!(mode.first_line("project:vega"), "/rk project:vega assess advisory:GHSA-jfh8-c2jp-5v3q");
    assert!(mode.works_on_a_project());
}

#[test]
fn an_assess_all_terminal_asks_for_every_advisory_of_the_project_without_naming_one() {
    let mode = TerminalMode::AssessAll;
    assert_eq!(mode.first_line("project:vega"), "/rk project:vega assess");
    assert!(mode.works_on_a_project() && !mode.follows_steps());
    assert_eq!(mode.to_string(), "ASSESS_ALL");
}
