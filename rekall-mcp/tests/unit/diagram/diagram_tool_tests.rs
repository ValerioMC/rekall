use serde_json::json;

use super::{diagram_id, sheets_argument, target_of};
use crate::protocol::ToolError;
use crate::tool::Anchor;
use rekall_service::diagram::DiagramTarget;

fn anchors(raw: &str) -> Vec<Anchor> {
    Anchor::parse_all(Some(raw)).unwrap()
}

#[test]
fn a_project_alone_is_a_project_target() {
    assert_eq!(target_of(&anchors("project:vega")).unwrap(), DiagramTarget::Project { label: "vega".into() });
}

#[test]
fn a_task_puts_the_diagram_on_its_project() {
    assert_eq!(
        target_of(&anchors("project:vega task:report")).unwrap(),
        DiagramTarget::Task { project_label: Some("vega".into()), task_label: "report".into() }
    );
    assert_eq!(target_of(&anchors("report")).unwrap(), DiagramTarget::Task { project_label: None, task_label: "report".into() });
}

#[test]
fn anything_but_one_project_or_a_task_is_refused() {
    assert!(matches!(target_of(&anchors("project:vega project:orion")), Err(ToolError::Failure(_))));
    assert!(matches!(target_of(&anchors("company:acme")), Err(ToolError::Failure(_))));
}

#[test]
fn sheets_sent_as_an_array_are_taken_as_they_are() {
    let sheets = json!([{"mermaid": "pie\n \"a\": 1"}]);
    assert_eq!(sheets_argument(Some(&sheets)).unwrap(), sheets);
}

#[test]
fn sheets_serialised_into_a_string_are_read_back_into_an_array() {
    let text = json!(r#"[{"mermaid":"pie"}]"#);
    assert_eq!(sheets_argument(Some(&text)).unwrap()[0]["mermaid"], json!("pie"));
}

#[test]
fn a_string_that_is_not_json_is_refused_as_an_argument() {
    assert!(matches!(sheets_argument(Some(&json!("sheets: []"))), Err(ToolError::Illegal(_))));
    assert!(matches!(sheets_argument(None), Err(ToolError::Illegal(_))));
}

#[test]
fn a_diagram_reference_has_to_be_an_id() {
    assert_eq!(diagram_id(None).unwrap(), None);
    let id = rekall_common::Id::random();
    assert_eq!(diagram_id(Some(format!(" {id} "))).unwrap(), Some(id));
    assert!(matches!(diagram_id(Some("order-flow".into())), Err(ToolError::Illegal(_))));
}
