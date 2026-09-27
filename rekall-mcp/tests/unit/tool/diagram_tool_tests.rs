use serde_json::json;

use super::{diagram_id, graph_document};
use crate::protocol::ToolError;

#[test]
fn a_graph_sent_as_an_object_is_taken_as_it_is() {
    let graph = json!({"format": "rekall.semantic-graph", "version": 1, "nodes": []});
    assert_eq!(graph_document(Some(&graph)).unwrap(), graph);
}

#[test]
fn a_graph_serialised_into_a_string_is_read_back_into_a_document() {
    let text = json!(r#"{"format":"rekall.semantic-graph","version":1,"nodes":[]}"#);
    assert_eq!(graph_document(Some(&text)).unwrap()["version"], json!(1));
}

#[test]
fn a_string_that_is_not_json_is_refused_as_an_argument() {
    assert!(matches!(graph_document(Some(&json!("nodes: []"))), Err(ToolError::Illegal(_))));
    assert!(matches!(graph_document(None), Err(ToolError::Illegal(_))));
}

#[test]
fn a_diagram_reference_has_to_be_an_id() {
    assert_eq!(diagram_id(None).unwrap(), None);
    let id = rekall_common::Id::random();
    assert_eq!(diagram_id(Some(format!(" {id} "))).unwrap(), Some(id));
    assert!(matches!(diagram_id(Some("order-flow".into())), Err(ToolError::Illegal(_))));
}
