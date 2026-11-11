use super::*;

fn parsed(json: &str) -> OpenTerminalRequest {
    serde_json::from_str(json).unwrap()
}

#[test]
fn a_request_without_a_mode_opens_a_work_terminal() {
    assert_eq!(parsed("{}").mode().unwrap(), TerminalMode::Work);
    assert_eq!(parsed(r#"{"mode":"PLAN"}"#).mode().unwrap(), TerminalMode::Plan);
    assert_eq!(parsed(r#"{"mode":"PLAN_FLOW"}"#).mode().unwrap(), TerminalMode::PlanFlow);
    assert_eq!(parsed(r#"{"mode":"REPLAN"}"#).mode().unwrap(), TerminalMode::Replan { message: None });
}

#[test]
fn a_replan_carries_the_request_as_what_the_plan_missed_unless_it_is_blank() {
    assert_eq!(
        parsed(r#"{"mode":"REPLAN","request":"cover the export"}"#).mode().unwrap(),
        TerminalMode::Replan { message: Some("cover the export".into()) }
    );
    assert_eq!(parsed(r#"{"mode":"REPLAN","request":"  "}"#).mode().unwrap(), TerminalMode::Replan { message: None });
}

#[test]
fn a_revise_request_evaluates_the_pending_thread_whatever_else_it_carries() {
    assert_eq!(parsed(r#"{"mode":"REVISE"}"#).mode().unwrap(), TerminalMode::Revise);
    assert_eq!(parsed(r#"{"mode":"REVISE","request":"ignored"}"#).mode().unwrap(), TerminalMode::Revise);
}

#[test]
fn a_rewrite_request_names_its_style_whatever_else_it_carries() {
    assert_eq!(parsed(r#"{"mode":"REWRITE_FIX"}"#).mode().unwrap(), TerminalMode::Rewrite { style: RewriteStyle::Fix });
    assert_eq!(
        parsed(r#"{"mode":"REWRITE_RESTRUCTURE","request":"ignored"}"#).mode().unwrap(),
        TerminalMode::Rewrite { style: RewriteStyle::Restructure }
    );
}

#[test]
fn a_generate_request_needs_something_to_generate() {
    assert!(parsed(r#"{"mode":"GENERATE","request":"  "}"#).mode().is_err());
    assert_eq!(
        parsed(r#"{"mode":"GENERATE","request":"the order flow"}"#).mode().unwrap(),
        TerminalMode::Generate { request: "the order flow".into(), kinds: vec![] }
    );
}

#[test]
fn generate_kinds_are_read_in_order_once_each_and_an_unknown_one_is_refused() {
    let asked = parsed(r#"{"mode":"GENERATE","request":"x","kinds":["state","sequence","state"]}"#);
    assert_eq!(asked.mode().unwrap(), TerminalMode::Generate { request: "x".into(), kinds: vec![DiagramKind::State, DiagramKind::Sequence] });
    assert!(parsed(r#"{"mode":"GENERATE","request":"x","kinds":["uml"]}"#).mode().is_err());
}

#[test]
fn an_explain_request_needs_something_to_explain() {
    assert!(parsed(r#"{"mode":"EXPLAIN"}"#).mode().is_err());
    assert_eq!(parsed(r#"{"mode":"EXPLAIN","request":"checkout"}"#).mode().unwrap(), TerminalMode::Explain { request: "checkout".into() });
}

#[test]
fn an_inventory_request_opens_an_inventory_terminal() {
    assert_eq!(parsed(r#"{"mode":"INVENTORY"}"#).mode().unwrap(), TerminalMode::Inventory);
}

#[test]
fn an_autoconfigure_request_opens_an_autoconfigure_terminal() {
    assert_eq!(parsed(r#"{"mode":"AUTOCONFIGURE"}"#).mode().unwrap(), TerminalMode::Autoconfigure);
}

#[test]
fn an_assess_request_names_the_advisory_by_its_ghsa_id() {
    assert_eq!(
        parsed(r#"{"mode":"ASSESS","advisory":"ghsa-JFH8-c2jp-5v3q"}"#).mode().unwrap(),
        TerminalMode::Assess { advisory: "GHSA-jfh8-c2jp-5v3q".into() }
    );
    assert!(parsed(r#"{"mode":"ASSESS"}"#).mode().is_err());
    assert!(parsed(r#"{"mode":"ASSESS","advisory":"GHSA-aaaa-aaaa-aaaa\r/exit"}"#).mode().is_err(), "only a GHSA id reaches the prompt");
}

#[test]
fn an_assess_all_request_names_no_advisory() {
    assert_eq!(parsed(r#"{"mode":"ASSESS_ALL","advisory":"GHSA-aaaa-aaaa-aaaa"}"#).mode().unwrap(), TerminalMode::AssessAll);
}

#[test]
fn the_command_carries_the_target_and_reads_a_missing_skip_as_no() {
    let target = TerminalTarget::Task(Id::random());
    let command = parsed(r#"{"model":"sonnet"}"#).command(target).unwrap();
    assert_eq!(command.target, target);
    assert!(!command.skip_permissions);
    assert_eq!(command.model.as_deref(), Some("sonnet"));
    assert_eq!(command.mode, TerminalMode::Work);
}

#[test]
fn a_request_without_a_kind_opens_a_terminal_session() {
    let target = TerminalTarget::Task(Id::random());

    assert_eq!(parsed("{}").command(target).unwrap().kind, SessionKind::Terminal);
    assert_eq!(parsed(r#"{"kind":"CHAT"}"#).command(target).unwrap().kind, SessionKind::Chat);
}
