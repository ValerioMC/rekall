use serde_json::json;

use super::{diagram_id, scenario_argument, sheets_argument, target_of};
use crate::protocol::ToolError;
use crate::tool::Anchor;
use rekall_core::diagram::DiagramTarget;
use rekall_core::ProjectRef;

fn anchors(raw: &str) -> Vec<Anchor> {
    Anchor::parse_all(Some(raw)).unwrap()
}

#[test]
fn a_project_alone_is_a_project_target() {
    assert_eq!(target_of(&anchors("project:vega")).unwrap(), DiagramTarget::Project { project: ProjectRef::bare("vega") });
}

#[test]
fn a_company_beside_the_project_qualifies_the_project_target() {
    assert_eq!(
        target_of(&anchors("company:Acme project:vega")).unwrap(),
        DiagramTarget::Project { project: ProjectRef::in_company("Acme", "vega") }
    );
}

#[test]
fn a_task_puts_the_diagram_on_its_project() {
    assert_eq!(
        target_of(&anchors("project:vega task:report")).unwrap(),
        DiagramTarget::Task { project_label: Some("vega".into()), task_code: "report".into() }
    );
    assert_eq!(target_of(&anchors("report")).unwrap(), DiagramTarget::Task { project_label: None, task_code: "report".into() });
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
    let id = rekall_kernel::Id::random();
    assert_eq!(diagram_id(Some(format!(" {id} "))).unwrap(), Some(id));
    assert!(matches!(diagram_id(Some("order-flow".into())), Err(ToolError::Illegal(_))));
}

#[test]
fn a_scenario_comes_as_an_object_or_a_string_holding_one() {
    let scenario = json!({"situation": "You drag step 4 above step 2", "example": "RK-12", "outcome": "1, 4, 2, 3"});
    assert_eq!(scenario_argument(Some(&scenario)).unwrap(), Some(scenario.clone()));
    assert_eq!(scenario_argument(Some(&json!(scenario.to_string()))).unwrap(), Some(scenario));
    assert_eq!(scenario_argument(None).unwrap(), None);
    assert!(matches!(scenario_argument(Some(&json!("drag a step"))), Err(ToolError::Illegal(_))));
}
