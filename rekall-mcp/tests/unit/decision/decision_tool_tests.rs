use serde_json::json;

use super::*;

#[test]
fn an_action_is_read_ignoring_case_and_anything_else_is_refused() {
    assert_eq!(Action::parse(" Correct ").unwrap(), Action::Correct);
    let refused = Action::parse("delete").unwrap_err();
    assert!(format!("{refused:?}").contains("ask, answer, correct, hold or remove"), "{refused:?}");
}

#[test]
fn an_argument_that_belongs_to_another_action_is_refused_by_name() {
    let sent = json!({ "anchors": "task:RK-1", "action": "hold", "key": "region", "note": "Asking", "answer": "eu" });

    let refused = refuse_stray_arguments(Some(&sent), Action::Hold).unwrap_err();

    assert!(format!("{refused:?}").contains("`hold` does not take 'answer'"), "{refused:?}");
}

#[test]
fn blank_and_null_arguments_are_not_stray() {
    let sent = json!({ "anchors": "task:RK-1", "action": "remove", "key": "region", "answer": " ", "detail": null });

    assert!(refuse_stray_arguments(Some(&sent), Action::Remove).is_ok());
}

#[test]
fn options_and_lines_arrive_as_lists_or_as_a_string_holding_one() {
    let options = json!([{ "label": "Vault", "effect": "Rotation", "recommended": true }, { "label": "Secret" }]);
    let lines = json!([{ "speaker": "Claude", "text": "Which store?" }, { "speaker": "person", "text": "Vault" }]);

    let expected = vec![
        OfferedOption { label: "Vault".into(), effect: Some("Rotation".into()), recommended: true },
        OfferedOption { label: "Secret".into(), effect: None, recommended: false },
    ];
    assert_eq!(options_of(Some(&options)).unwrap(), expected);
    assert_eq!(options_of(Some(&json!(options.to_string()))).unwrap(), expected);
    assert_eq!(lines_of(Some(&lines)).unwrap().iter().map(|line| line.speaker).collect::<Vec<_>>(), [Speaker::Claude, Speaker::Person]);
    assert!(options_of(None).unwrap().is_empty());
}

#[test]
fn a_malformed_list_or_an_unknown_speaker_is_refused() {
    assert!(format!("{:?}", options_of(Some(&json!([{ "effect": "x" }]))).unwrap_err()).contains("'options' is not a list"));
    assert!(format!("{:?}", lines_of(Some(&json!([{ "speaker": "robot", "text": "x" }]))).unwrap_err()).contains("`claude` or `person`"));
}

#[test]
fn a_session_records_from_the_terminal_or_the_chat_and_never_as_the_person() {
    assert_eq!(source_of(None).unwrap(), DecisionSource::Terminal);
    assert_eq!(source_of(Some("CHAT")).unwrap(), DecisionSource::Chat);
    assert!(source_of(Some("person")).is_err());
}

#[test]
fn a_step_is_a_number() {
    assert_eq!(step_number_of(" 3 ").unwrap(), 3);
    assert!(step_number_of("three").is_err());
}
