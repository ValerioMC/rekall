use super::*;
use crate::format::GraphCodec;

#[test]
fn a_line_resolves_to_every_node_covering_it_narrowest_first() {
    let graph = GraphCodec::parse(
        r#"{"format":"rekall.semantic-graph","version":1,"nodes":[
            {"id":"fn","kind":"code","title":"process_order()","sources":[{"file":"src/order.rs","startLine":10,"endLine":200}]},
            {"id":"check","kind":"decision","title":"Paid?","sources":[{"file":"./src/order.rs","startLine":40,"endLine":52},{"file":"src/pay.rs","startLine":1,"endLine":9}]},
            {"id":"module","kind":"concept","title":"Ordering","sources":[{"file":"src/order.rs"}]},
            {"id":"elsewhere","kind":"action","title":"Notify","sources":[{"file":"src/notify.rs","startLine":40,"endLine":52}]}]}"#,
    )
    .unwrap();

    let hits = TraceIndex::of(&graph).nodes_at("src/order.rs", 45);
    let ids: Vec<&str> = hits.iter().map(|hit| hit.node_id.as_str()).collect();
    assert_eq!(ids, vec!["check", "fn", "module"]);
    assert_eq!(hits[0].span_lines, Some(13));
    assert_eq!(hits[2].span_lines, None);

    assert!(TraceIndex::of(&graph).nodes_at("src/order.rs", 5).iter().all(|hit| hit.node_id == "module"));
}
