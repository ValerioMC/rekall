use super::*;
use crate::graph::{NodeKind, RelationKind};

fn document(edges: &str) -> String {
    format!(
        r#"{{"format":"rekall.semantic-graph","version":1,
            "nodes":[{{"id":" receive ","kind":"action","title":"  Receive order "}},
                     {{"id":"paid","kind":"decision","title":"Paid?","confidence":0.8,"provenance":"inferred"}}],
            "edges":{edges}}}"#
    )
}

#[test]
fn a_document_reads_trimmed_and_numbers_edges_that_arrived_without_an_id() {
    let graph = GraphCodec::parse(&document(
        r#"[{"from":"receive","to":"paid","relation":"leads_to"},{"id":"e1","from":"paid","to":"receive","relation":"conditionally_leads_to","label":"no"}]"#,
    ))
    .unwrap();

    assert_eq!(graph.nodes[0].id, "receive");
    assert_eq!(graph.nodes[0].title, "Receive order");
    assert_eq!(graph.nodes[1].kind, NodeKind::Decision);
    assert_eq!(graph.edges[0].id, "e2", "e1 is taken by the second edge");
    assert_eq!(graph.edges[1].relation, RelationKind::ConditionallyLeadsTo);
}

#[test]
fn what_it_writes_it_reads_back_identically() {
    let graph = GraphCodec::parse(&document("[]")).unwrap();
    assert_eq!(GraphCodec::parse(&GraphCodec::write(&graph)).unwrap(), graph);
}

#[test]
fn another_format_or_version_is_refused_by_name() {
    let foreign = document("[]").replace("rekall.semantic-graph", "mermaid");
    assert_eq!(GraphCodec::parse(&foreign), Err(GraphFormatError::UnknownFormat("mermaid".into())));

    let future = document("[]").replace("\"version\":1", "\"version\":2");
    assert_eq!(GraphCodec::parse(&future), Err(GraphFormatError::UnsupportedVersion(2)));
}

#[test]
fn a_field_the_format_does_not_define_is_refused_rather_than_dropped() {
    let text = document("[]").replace("\"title\":\"Paid?\"", "\"title\":\"Paid?\",\"colour\":\"red\"");
    assert!(matches!(GraphCodec::parse(&text), Err(GraphFormatError::Malformed(detail)) if detail.contains("colour")));
}
