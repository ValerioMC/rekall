use serde_json::{json, Value};

use super::*;
use crate::format::DocumentCodec;

fn document(sheets: Value) -> DiagramDocument {
    DocumentCodec::from_value(json!({ "sheets": sheets })).unwrap()
}

fn paths(result: Result<(), DocumentViolations>) -> Vec<String> {
    result.unwrap_err().0.into_iter().map(|violation| violation.path).collect()
}

const CHECKOUT: &str = "sequenceDiagram\n  participant S as OrderService\n  U->>S: place\n  S-->>U: done";

#[test]
fn a_sheet_linking_its_participants_and_messages_passes() {
    let valid = document(json!([{
        "title": "Checkout",
        "mermaid": CHECKOUT,
        "links": {
            "S": [{ "file": "src/order.rs", "startLine": 10, "endLine": 80, "symbol": "OrderService" }],
            "#2": [{ "file": "src/order.rs", "startLine": 70 }]
        }
    }]));
    assert_eq!(DocumentValidator::validate(&valid), Ok(()));
}

#[test]
fn a_diagram_has_between_one_and_eight_sheets() {
    assert_eq!(paths(DocumentValidator::validate(&document(json!([])))), vec!["sheets"]);
    let nine: Vec<Value> = (0..9).map(|_| json!({ "mermaid": "pie\n \"a\": 1" })).collect();
    assert_eq!(paths(DocumentValidator::validate(&document(Value::Array(nine)))), vec!["sheets"]);
}

#[test]
fn the_mermaid_has_to_declare_a_known_type() {
    let broken = document(json!([{ "mermaid": "  " }, { "mermaid": "A --> B" }]));
    assert_eq!(paths(DocumentValidator::validate(&broken)), vec!["sheets[0].mermaid", "sheets[1].mermaid"]);
}

#[test]
fn every_link_has_to_name_something_the_sheet_draws() {
    let broken = document(json!([{
        "mermaid": CHECKOUT,
        "links": {
            "Ghost": [{ "file": "a.rs" }],
            "#3": [{ "file": "a.rs" }],
            "#x": [{ "file": "a.rs" }]
        }
    }]));
    assert_eq!(
        paths(DocumentValidator::validate(&broken)),
        vec![r##"sheets[0].links["#3"]"##, r##"sheets[0].links["#x"]"##, r#"sheets[0].links["Ghost"]"#]
    );
}

#[test]
fn message_numbers_belong_to_sequences_only() {
    let broken = document(json!([{ "mermaid": "flowchart LR\n A-->B", "links": { "#1": [{ "file": "a.rs" }] } }]));
    assert_eq!(paths(DocumentValidator::validate(&broken)), vec![r##"sheets[0].links["#1"]"##]);
}

#[test]
fn a_type_mermaid_draws_by_itself_carries_no_links() {
    let broken = document(json!([{ "mermaid": "mindmap\n  root", "links": { "root": [{ "file": "a.rs" }] } }]));
    assert_eq!(paths(DocumentValidator::validate(&broken)), vec!["sheets[0].links"]);
}

#[test]
fn spans_are_checked_where_they_sit() {
    let broken = document(json!([{
        "mermaid": "flowchart LR\n A-->B",
        "links": {
            "A": [{ "file": " ", "startLine": 0 }, { "file": "a.rs", "endLine": 3 }],
            "B": []
        }
    }]));
    assert_eq!(
        paths(DocumentValidator::validate(&broken)),
        vec![
            r#"sheets[0].links["A"][0].file"#,
            r#"sheets[0].links["A"][0].startLine"#,
            r#"sheets[0].links["A"][1].endLine"#,
            r#"sheets[0].links["B"]"#
        ]
    );
}
