use super::*;
use crate::format::GraphCodec;

fn graph(nodes: &str, edges: &str) -> SemanticGraph {
    GraphCodec::parse(&format!(r#"{{"format":"rekall.semantic-graph","version":1,"nodes":{nodes},"edges":{edges}}}"#)).unwrap()
}

const THREE: &str = r#"[{"id":"fn","kind":"code","title":"process_order()","sources":[{"file":"src/order.rs","startLine":10,"endLine":200}]},
                        {"id":"a","kind":"action","title":"Validate"},
                        {"id":"b","kind":"state","title":"Paid"}]"#;

fn paths(result: Result<(), GraphViolations>) -> Vec<String> {
    result.unwrap_err().0.into_iter().map(|violation| violation.path).collect()
}

#[test]
fn a_well_formed_graph_passes() {
    let valid = graph(THREE, r#"[{"from":"fn","to":"a","relation":"contains"},{"from":"a","to":"b","relation":"transitions_to"}]"#);
    assert_eq!(GraphValidator::validate(&valid), Ok(()));
}

#[test]
fn every_broken_rule_is_reported_with_its_path() {
    let broken = graph(
        r#"[{"id":"a","kind":"action","title":"","confidence":1.5},{"id":"a","kind":"Weird Kind","title":"x","sources":[{"file":"f.rs","startLine":9,"endLine":3}]}]"#,
        r#"[{"from":"a","to":"ghost","relation":"calls"}]"#,
    );
    assert_eq!(
        paths(GraphValidator::validate(&broken)),
        vec!["nodes[0].title", "nodes[0].confidence", "nodes[1].kind", "nodes[1].sources[0].endLine", "nodes[1].id", "edges[0].to"]
    );
}

#[test]
fn contains_has_to_form_a_forest() {
    let two_parents = graph(THREE, r#"[{"from":"fn","to":"b","relation":"contains"},{"from":"a","to":"b","relation":"contains"}]"#);
    assert_eq!(paths(GraphValidator::validate(&two_parents)), vec!["edges[1]"]);

    let cycle = graph(THREE, r#"[{"from":"a","to":"b","relation":"contains"},{"from":"b","to":"a","relation":"contains"}]"#);
    assert_eq!(paths(GraphValidator::validate(&cycle)), vec!["edges"]);
}

#[test]
fn an_edge_repeated_exactly_is_refused_but_a_different_label_is_another_edge() {
    let repeated = graph(THREE, r#"[{"from":"a","to":"b","relation":"leads_to"},{"from":"a","to":"b","relation":"leads_to"}]"#);
    assert_eq!(paths(GraphValidator::validate(&repeated)), vec!["edges[1]"]);

    let labelled = graph(THREE, r#"[{"from":"a","to":"b","relation":"conditionally_leads_to","label":"paid"},{"from":"a","to":"b","relation":"conditionally_leads_to","label":"refunded"}]"#);
    assert_eq!(GraphValidator::validate(&labelled), Ok(()));
}

#[test]
fn an_empty_graph_is_refused() {
    assert_eq!(paths(GraphValidator::validate(&graph("[]", "[]"))), vec!["nodes"]);
}
