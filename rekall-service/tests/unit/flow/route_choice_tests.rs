use rekall_common::{Id, Instant};

use super::*;

fn routes(outcomes: &[&str]) -> Vec<step_link::Model> {
    let from = Id::random();
    outcomes
        .iter()
        .enumerate()
        .map(|(at, outcome)| step_link::Model::new(Id::random(), from, Id::random(), (*outcome).into(), at as i32, Instant::now()))
        .collect()
}

#[test]
fn the_named_outcome_picks_its_route() {
    let outgoing = routes(&["passed", "app-error"]);
    let chosen = choose_route("Test", &outgoing, Some("app-error")).unwrap().unwrap();
    assert_eq!(chosen.outcome, "app-error");
}

#[test]
fn an_outcome_with_no_route_is_refused_listing_the_ones_there_are() {
    let outgoing = routes(&["passed", "app-error"]);
    let error = choose_route("Test", &outgoing, Some("flaky")).unwrap_err().to_string();
    assert!(error.contains("`passed`, `app-error`"), "{error}");
}

#[test]
fn a_single_route_is_taken_without_an_outcome() {
    let outgoing = routes(&["deployed"]);
    assert_eq!(choose_route("Deploy", &outgoing, None).unwrap().unwrap().outcome, "deployed");
}

#[test]
fn several_routes_and_no_outcome_is_refused() {
    let outgoing = routes(&["passed", "app-error"]);
    assert!(choose_route("Test", &outgoing, None).is_err());
}

#[test]
fn a_step_with_no_routes_ends_the_flow() {
    assert!(choose_route("Ship", &[], None).unwrap().is_none());
}

#[test]
fn an_outcome_on_a_step_that_ends_the_flow_is_refused() {
    let error = choose_route("Ship", &[], Some("passed")).unwrap_err().to_string();
    assert!(error.contains("ends the flow"), "{error}");
}
