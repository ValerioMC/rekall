use super::*;

#[test]
fn every_kind_reads_back_from_its_name() {
    for kind in DiagramKind::ALL {
        assert_eq!(DiagramKind::of_name(kind.name()), Some(kind));
    }
    assert_eq!(DiagramKind::of_name("Sequence"), None);
}

#[test]
fn a_keyword_and_its_variants_name_one_kind() {
    assert_eq!(DiagramKind::of_keyword("graph"), Some(DiagramKind::Flowchart));
    assert_eq!(DiagramKind::of_keyword("stateDiagram-v2"), Some(DiagramKind::State));
    assert_eq!(DiagramKind::of_keyword("C4Dynamic"), Some(DiagramKind::C4));
    assert_eq!(DiagramKind::of_keyword("sequencediagram"), None);
}

#[test]
fn only_the_kinds_the_console_draws_link_elements() {
    let linking: Vec<DiagramKind> = DiagramKind::ALL.into_iter().filter(|kind| kind.links_elements()).collect();
    assert_eq!(linking, vec![DiagramKind::Sequence, DiagramKind::Flowchart, DiagramKind::State, DiagramKind::Class, DiagramKind::Er]);
}

#[test]
fn kinds_serialise_as_their_names() {
    assert_eq!(serde_json::to_string(&DiagramKind::Er).unwrap(), "\"er\"");
    assert_eq!(serde_json::to_string(&DiagramKind::Gitgraph).unwrap(), "\"gitgraph\"");
}
