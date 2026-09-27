use super::*;

#[test]
fn a_known_name_reads_as_its_kind_and_writes_back_unchanged() {
    for kind in NodeKind::KNOWN {
        let read = NodeKind::from(kind.name().to_string());
        assert_eq!(read, kind);
        assert!(!read.is_custom());
        assert_eq!(String::from(read), kind.name());
    }
}

#[test]
fn an_unknown_name_is_kept_as_a_custom_kind() {
    let kind: NodeKind = serde_json::from_str("\"queue\"").unwrap();
    assert_eq!(kind, NodeKind::Custom("queue".into()));
    assert_eq!(serde_json::to_string(&kind).unwrap(), "\"queue\"");
}
