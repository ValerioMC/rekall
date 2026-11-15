use super::*;

#[test]
fn a_skill_is_a_name_with_optional_arguments() {
    let request: SkillAssignmentRequest = serde_json::from_str(r#"{"skills":[{"name":"impeccable","arguments":"redesign website ui"},{"name":"rk"}]}"#).unwrap();

    let commands = request.into_commands();

    assert_eq!(commands, [
        SkillCommand { name: "impeccable".into(), arguments: Some("redesign website ui".into()) },
        SkillCommand { name: "rk".into(), arguments: None },
    ]);
}

#[test]
fn the_plain_string_shape_is_no_longer_read() {
    assert!(serde_json::from_str::<SkillAssignmentRequest>(r#"{"skills":["impeccable"]}"#).is_err());
}

#[test]
fn a_missing_list_is_an_empty_one() {
    assert!(serde_json::from_str::<SkillAssignmentRequest>("{}").unwrap().into_commands().is_empty());
}
