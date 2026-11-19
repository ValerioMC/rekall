use super::*;
use crate::flow::PlannedRoute;

fn step(title: &str, routes: &[(&str, Option<usize>)]) -> PlannedStep {
    PlannedStep {
        title: title.into(),
        detail: None,
        routes: routes.iter().map(|(outcome, to)| PlannedRoute { outcome: (*outcome).into(), to: *to }).collect(),
        max_visits: None,
        starts_session: false,
    }
}

/// Build, deploy, test; a failed test goes to a fix that goes back to deploy, a pass ends it.
fn pipeline() -> FlowPlan {
    FlowPlan::new(vec![
        step("Build", &[("built", Some(1))]),
        step("Deploy", &[("deployed", Some(2))]),
        step("Test", &[("passed", None), ("app-error", Some(3))]),
        step("Fix", &[("fixed", Some(1))]),
    ])
}

fn refusal(plan: FlowPlan) -> String {
    plan.check().unwrap_err().to_string()
}

#[test]
fn a_flow_with_a_loop_and_a_way_out_is_accepted() {
    let plan = pipeline();

    plan.check().unwrap();
    assert_eq!(plan.route_count(), 5);
}

#[test]
fn a_last_step_with_no_routes_ends_the_flow() {
    FlowPlan::new(vec![step("Build", &[("built", Some(1))]), step("Ship", &[])]).check().unwrap();
}

#[test]
fn a_plan_without_a_single_route_is_a_checklist() {
    let error = refusal(FlowPlan::new(vec![step("Build", &[]), step("Ship", &[])]));

    assert!(error.contains("rekall_propose_step"), "{error}");
}

#[test]
fn an_empty_or_oversized_plan_is_refused() {
    assert!(refusal(FlowPlan::new(Vec::new())).contains("at least one step"));
    let many = (0..=PROPOSED_DRAFTS_MAX).map(|at| step(&format!("Step {at}"), &[])).collect();
    assert!(refusal(FlowPlan::new(many)).contains("at most"));
}

#[test]
fn a_title_twice_ignoring_case_is_refused() {
    let error = refusal(FlowPlan::new(vec![step("Build", &[("built", Some(1))]), step("build ", &[])]));

    assert!(error.contains("appears twice"), "{error}");
}

#[test]
fn an_outcome_twice_on_one_step_is_refused_once_normalised() {
    let error = refusal(FlowPlan::new(vec![
        step("Test", &[("App error", Some(1)), ("app-error", None)]),
        step("Fix", &[]),
    ]));

    assert!(error.contains("`app-error` twice"), "{error}");
}

#[test]
fn a_route_onto_itself_or_past_the_plan_is_refused() {
    assert!(refusal(FlowPlan::new(vec![step("Test", &[("again", Some(0))])])).contains("routes to itself"));
    assert!(refusal(FlowPlan::new(vec![step("Test", &[("next", Some(4))])])).contains("to step 5"));
}

#[test]
fn a_step_the_first_never_leads_to_is_named() {
    let error = refusal(FlowPlan::new(vec![
        step("Build", &[("built", None)]),
        step("Orphan", &[("done", Some(0))]),
    ]));

    assert!(error.contains("never reaches 'Orphan'"), "{error}");
}

#[test]
fn a_loop_with_no_way_out_names_its_steps() {
    let error = refusal(FlowPlan::new(vec![
        step("Deploy", &[("deployed", Some(1))]),
        step("Test", &[("app-error", Some(0))]),
    ]));

    assert!(error.contains("loop forever through 'Deploy', 'Test'"), "{error}");
}

#[test]
fn too_many_routes_out_of_one_step_is_refused() {
    let targets: Vec<(String, Option<usize>)> = (0..=ROUTES_PER_STEP_MAX).map(|at| (format!("o{at}"), None)).collect();
    let routes: Vec<(&str, Option<usize>)> = targets.iter().map(|(outcome, to)| (outcome.as_str(), *to)).collect();

    assert!(refusal(FlowPlan::new(vec![step("Decide", &routes)])).contains("Split it"));
}

#[test]
fn the_shape_holds_for_a_graph_that_cannot_run_yet() {
    let unfinished = FlowPlan::new(vec![step("Build", &[("built", None)]), step("Orphan", &[])]);

    unfinished.check_shape().unwrap();
    assert!(unfinished.check().is_err());
}

#[test]
fn the_shape_still_refuses_what_no_flow_could_hold() {
    assert!(FlowPlan::new(vec![step("Test", &[("again", Some(0))])]).check_shape().is_err());
    assert!(FlowPlan::new(vec![step("Test", &[]), step("test", &[])]).check_shape().is_err());
}

#[test]
fn a_plan_that_follows_kept_steps_counts_its_routes_from_its_own_first_step() {
    let plan = FlowPlan::new(vec![step("Build", &[("built", Some(3))]), step("Ship", &[])]);

    let following = plan.following(&["Old one", "Old two"]).unwrap();

    assert_eq!(following.steps[0].routes[0].to, Some(1));
    following.check().unwrap();
}

#[test]
fn a_route_into_a_kept_step_is_refused_naming_it() {
    let plan = FlowPlan::new(vec![step("Build", &[("built", Some(1))]), step("Ship", &[])]);

    let error = plan.following(&["Old one", "Old two"]).unwrap_err().to_string();

    assert!(error.contains("to step 2, 'Old two', which stays as it is"), "{error}");
}

#[test]
fn a_plan_following_kept_steps_quotes_whole_checklist_numbers_in_a_refusal() {
    let plan = FlowPlan::new(vec![step("Build", &[("built", Some(9))])]).following(&["Old one"]).unwrap();

    let error = plan.check().unwrap_err().to_string();

    assert!(error.contains("to step 10, and the flow has 2 steps"), "{error}");
}

#[test]
fn the_end_of_the_flow_stays_a_valid_target_after_kept_steps() {
    let plan = FlowPlan::new(vec![step("Build", &[("built", None)])]).following(&["Old one"]).unwrap();

    plan.check().unwrap();
}
