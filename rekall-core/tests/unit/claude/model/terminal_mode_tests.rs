use rekall_diagram::DiagramKind;

use super::*;

const ANCHORS: &str = "task:REPORT-BUILDER";

#[test]
fn a_work_terminal_loads_the_task() {
    assert_eq!(TerminalMode::Work.first_line(ANCHORS), "/rk task:REPORT-BUILDER");
}

#[test]
fn a_plan_terminal_asks_the_session_to_plan_the_task() {
    assert_eq!(TerminalMode::Plan.first_line(ANCHORS), "/rk task:REPORT-BUILDER plan");
}

#[test]
fn a_plan_flow_terminal_asks_the_session_for_a_flow_and_follows_no_step() {
    assert_eq!(TerminalMode::PlanFlow.first_line(ANCHORS), "/rk task:REPORT-BUILDER plan flow");
    assert!(!TerminalMode::PlanFlow.follows_steps());
}

#[test]
fn a_replan_terminal_asks_the_session_to_redesign_the_steps_and_follows_no_step() {
    let mode = TerminalMode::Replan { message: None };
    assert_eq!(mode.first_line(ANCHORS), "/rk task:REPORT-BUILDER replan");
    assert!(!mode.follows_steps());
    assert_eq!(mode.to_string(), "REPLAN");
}

#[test]
fn a_revise_terminal_asks_the_session_to_evaluate_the_pending_thread_and_follows_no_step() {
    assert_eq!(TerminalMode::Revise.first_line(ANCHORS), "/rk task:REPORT-BUILDER revise");
    assert!(!TerminalMode::Revise.follows_steps() && !TerminalMode::Revise.works_on_a_project());
    assert_eq!(TerminalMode::Revise.to_string(), "REVISE");
}

#[test]
fn a_rewrite_terminal_names_the_style_and_follows_no_step() {
    let fix = TerminalMode::Rewrite { style: RewriteStyle::Fix };
    let restructure = TerminalMode::Rewrite { style: RewriteStyle::Restructure };
    assert_eq!(fix.first_line(ANCHORS), "/rk task:REPORT-BUILDER rewrite fix");
    assert_eq!(restructure.first_line(ANCHORS), "/rk task:REPORT-BUILDER rewrite restructure");
    assert!(!fix.follows_steps() && !fix.works_on_a_project());
    assert_eq!((fix.to_string(), restructure.to_string()), ("REWRITE_FIX".to_string(), "REWRITE_RESTRUCTURE".to_string()));
}

#[test]
fn a_replan_terminal_carries_what_the_plan_missed_as_one_quoted_line() {
    let mode = TerminalMode::Replan { message: Some("Also cover \"export\"\nto csv  ".into()) };
    assert_eq!(mode.first_line(ANCHORS), "/rk task:REPORT-BUILDER replan \"Also cover 'export' to csv\"");
}

#[test]
fn a_generate_terminal_carries_the_request_as_one_quoted_line() {
    let mode = TerminalMode::Generate { request: "How does an order\nget \"paid\"?  ".into(), kinds: vec![] };
    assert_eq!(mode.first_line(ANCHORS), "/rk task:REPORT-BUILDER generate \"How does an order get 'paid'?\"");
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

#[test]
fn an_autoconfigure_terminal_works_on_the_project_and_follows_no_step() {
    let mode = TerminalMode::Autoconfigure;
    assert_eq!(mode.first_line("project:lightcraft"), "/rk project:lightcraft autoconfigure");
    assert!(mode.works_on_a_project() && !mode.follows_steps());
    assert_eq!(mode.to_string(), "AUTOCONFIGURE");
}

#[test]
fn a_mode_goes_to_the_console_under_its_display_name_whatever_it_carries() {
    let generate = TerminalMode::Generate { request: "how checkout charges".into(), kinds: Vec::new() };
    assert_eq!(serde_json::to_string(&generate).unwrap(), "\"GENERATE\"");
    assert_eq!(serde_json::to_string(&TerminalMode::Autoconfigure).unwrap(), "\"AUTOCONFIGURE\"");
}

#[test]
fn a_merge_terminal_asks_the_session_to_resolve_the_branch_and_follows_no_step() {
    assert_eq!(TerminalMode::Merge.first_line(ANCHORS), "/rk task:REPORT-BUILDER merge");
    assert!(!TerminalMode::Merge.follows_steps() && !TerminalMode::Merge.works_on_a_project());
    assert_eq!(TerminalMode::Merge.to_string(), "MERGE");
}

#[test]
fn a_pull_request_merge_terminal_names_the_branch_the_pull_request_targets() {
    let mode = TerminalMode::MergePullRequest { base: "main".into() };

    assert_eq!(mode.first_line(ANCHORS), "/rk task:REPORT-BUILDER merge pull-request base:main");
    assert!(!mode.follows_steps() && !mode.works_on_a_project());
    assert_eq!(mode.to_string(), "MERGE");
}
