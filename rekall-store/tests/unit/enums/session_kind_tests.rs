use super::SessionKind;

#[test]
fn a_session_is_a_terminal_unless_chosen_otherwise() {
    assert_eq!(SessionKind::default(), SessionKind::Terminal);
}

#[test]
fn the_kind_is_written_and_read_in_capitals() {
    assert_eq!(serde_json::to_string(&SessionKind::Chat).unwrap(), "\"CHAT\"");
    assert_eq!(serde_json::from_str::<SessionKind>("\"TERMINAL\"").unwrap(), SessionKind::Terminal);
    assert_eq!(SessionKind::Chat.to_string(), "CHAT");
}
