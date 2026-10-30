use super::*;

fn read(json: &str) -> RunQueueSettingsCommand {
    serde_json::from_str::<SettingsRequest>(json).unwrap().into()
}

#[test]
fn a_request_without_a_session_kind_asks_for_terminals() {
    assert_eq!(read(r#"{"ceilingPercent":80}"#).session_kind, SessionKind::Terminal);
}

#[test]
fn a_request_names_the_session_kind_in_capitals() {
    let command = read(r#"{"ceilingPercent":null,"skipPermissions":true,"model":"sonnet","sessionKind":"CHAT"}"#);

    assert_eq!(command.session_kind, SessionKind::Chat);
    assert!(command.skip_permissions);
    assert_eq!(command.model.as_deref(), Some("sonnet"));
}

#[test]
fn a_session_kind_that_is_not_one_is_refused() {
    assert!(serde_json::from_str::<SettingsRequest>(r#"{"sessionKind":"TUI"}"#).is_err());
}
