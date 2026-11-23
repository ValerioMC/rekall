use super::*;
use crate::decision::enums::DecisionSource;

fn a_decision() -> Model {
    Model::new(Id::random(), "db-password".into(), "Where does the password live?".into(), DecisionSource::Terminal, Instant::now())
}

#[test]
fn a_new_decision_is_open_and_about_the_task() {
    let decision = a_decision();
    assert_eq!((decision.state, decision.step_number, decision.answer.as_deref()), (DecisionState::Open, None, None));
    assert!(decision.validate().is_ok());
}

#[test]
fn a_key_that_is_not_a_slug_is_refused() {
    let mut decision = a_decision();
    decision.key = "DB password".into();
    assert!(decision.validate().unwrap_err().message().contains("key must be a slug"));
}

#[test]
fn a_step_number_below_one_is_refused() {
    let mut decision = a_decision();
    decision.step_number = Some(0);
    assert!(decision.validate().unwrap_err().message().contains("stepNumber"));
}

#[test]
fn text_over_its_cap_is_refused() {
    let mut decision = a_decision();
    decision.question = "q".repeat(QUESTION_MAX_CHARACTERS + 1);
    decision.why = Some("w".repeat(WHY_MAX_CHARACTERS + 1));
    decision.answer = Some("a".repeat(ANSWER_MAX_CHARACTERS + 1));
    let message = decision.validate().unwrap_err().message().to_string();
    assert!(message.contains("question is capped at 500") && message.contains("why is capped at 2000") && message.contains("answer is capped at 4000"), "{message}");
}

#[test]
fn changing_an_answer_keeps_the_one_it_replaced_and_when() {
    let mut decision = a_decision();
    decision.answer = Some("a Kubernetes secret".into());
    let later = Instant::now();
    decision.change_answer("a sealed secret".into(), later);
    assert_eq!(decision.previous_answer.as_deref(), Some("a Kubernetes secret"));
    assert_eq!((decision.answer.as_deref(), decision.changed_at), (Some("a sealed secret"), Some(later)));
}
