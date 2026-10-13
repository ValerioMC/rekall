use super::*;

fn route(outcome: &str) -> Model {
    Model::new(Id::random(), Id::random(), Id::random(), outcome.into(), 0, Instant::now())
}

#[test]
fn a_route_between_two_steps_with_a_slug_outcome_is_valid() {
    assert!(route("app-error").validate().is_ok());
}

#[test]
fn a_route_onto_its_own_step_is_refused() {
    let mut looped = route("retry");
    looped.to_step_id = looped.from_step_id;
    let error = looped.validate().unwrap_err();
    assert!(error.to_string().contains("cannot route to itself"));
}

#[test]
fn an_outcome_that_is_not_a_slug_is_refused() {
    assert!(route("App Error").validate().is_err());
}

#[test]
fn an_outcome_past_its_cap_is_refused() {
    assert!(route(&"a".repeat(OUTCOME_MAX_CHARACTERS + 1)).validate().is_err());
}

#[test]
fn a_typed_outcome_is_normalised_like_a_label() {
    assert_eq!(normalise_outcome(Some("  Pipeline Error ")).unwrap(), "pipeline-error");
}

#[test]
fn an_outcome_with_nothing_usable_is_refused_naming_examples() {
    let error = normalise_outcome(Some(" !! ")).unwrap_err();
    assert!(error.to_string().contains("passed"));
}
