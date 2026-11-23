use super::*;

#[test]
fn the_body_maps_onto_the_command_with_camel_case_names() {
    let request: AddDecisionRequest = serde_json::from_str(r#"{"topic":"Cluster","question":"Which namespace?","answer":"billing","stepNumber":2}"#).unwrap();

    let command = request.into_command();

    assert_eq!((command.topic.as_deref(), command.question.as_deref(), command.answer.as_deref(), command.step_number), (Some("Cluster"), Some("Which namespace?"), Some("billing"), Some(2)));
}

#[test]
fn a_body_with_nothing_in_it_reads_as_absent_fields() {
    let command = serde_json::from_str::<AddDecisionRequest>("{}").unwrap().into_command();

    assert_eq!((command.question, command.answer, command.step_number), (None, None, None));
}
