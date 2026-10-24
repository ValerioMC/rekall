use super::*;

fn route(outcome: &str, to_key: Option<&str>) -> GraphRouteRequest {
    GraphRouteRequest { outcome: Some(outcome.into()), to_key: to_key.map(str::to_string) }
}

fn step(key: &str, title: &str, routes: Vec<GraphRouteRequest>) -> GraphStepRequest {
    GraphStepRequest { key: Some(key.into()), title: Some(title.into()), routes: Some(routes), ..Default::default() }
}

#[test]
fn routes_point_at_the_position_of_the_step_their_key_names() {
    let draft = GraphDraft::of(&[
        step("push", "Push commit", vec![route("pushed", Some("check"))]),
        step("check", "Check", vec![route("Error", Some("push")), route("healthy", None)]),
    ])
    .unwrap();

    let check = &draft.plan.steps[1];
    assert_eq!(draft.plan.steps[0].routes[0].to, Some(1));
    assert_eq!((check.routes[0].outcome.as_str(), check.routes[0].to), ("error", Some(0)), "the outcome is stored as a slug");
    assert_eq!(check.routes[1].to, None, "no key leads to the end");
}

#[test]
fn a_route_to_a_key_the_graph_lacks_is_refused() {
    let refused = GraphDraft::of(&[step("push", "Push", vec![route("pushed", Some("deploy"))])]).unwrap_err();

    assert!(refused.to_string().contains("'deploy', which is not a step"), "{refused}");
}

#[test]
fn a_key_names_one_step_and_every_step_has_one() {
    let twice = GraphDraft::of(&[step("a", "Push", vec![]), step("a", "Build", vec![])]).unwrap_err();
    let missing = GraphDraft::of(&[GraphStepRequest { title: Some("Push".into()), ..Default::default() }]).unwrap_err();

    assert!(twice.to_string().contains("names two steps"), "{twice}");
    assert!(missing.to_string().contains("no key"), "{missing}");
}

#[test]
fn a_graph_that_cannot_run_yet_is_still_a_draft() {
    let draft = GraphDraft::of(&[step("push", "Push", vec![route("pushed", None)]), step("orphan", "Orphan", vec![])]);

    assert!(draft.is_ok());
}

#[test]
fn what_no_flow_could_hold_is_refused() {
    assert!(GraphDraft::of(&[]).is_err(), "no steps");
    assert!(GraphDraft::of(&[step("a", "Test", vec![route("again", Some("a"))])]).is_err(), "a route onto its own step");
    let budget = GraphStepRequest { max_visits: Some(0), ..step("a", "Push", vec![]) };
    assert!(GraphDraft::of(&[budget]).is_err(), "a visit budget out of range");
}

#[test]
fn only_steps_that_reach_a_connection_are_listed_as_reaching_one() {
    let cluster = Id::random();
    let deploy = GraphStepRequest { connection_id: Some(cluster), ..step("deploy", "Deploy", vec![]) };
    let draft = GraphDraft::of(&[step("push", "Push", vec![]), deploy]).unwrap();

    let reaching: Vec<(String, Id)> = draft.reaching().map(|(step, id)| (step.title.clone(), id)).collect();
    assert_eq!(reaching, vec![("Deploy".to_string(), cluster)]);
}
