use super::*;

#[test]
fn an_option_over_its_caps_is_refused() {
    let mut option = Model::new(Id::random(), 1, "l".repeat(LABEL_MAX_CHARACTERS + 1));
    option.effect = Some("e".repeat(EFFECT_MAX_CHARACTERS + 1));
    let message = option.validate().unwrap_err().message().to_string();
    assert!(message.contains("label is capped at 200") && message.contains("effect is capped at 1000"), "{message}");
}

#[test]
fn an_option_with_a_label_passes() {
    assert!(Model::new(Id::random(), 1, "Sealed secret".into()).validate().is_ok());
}
