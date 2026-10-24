use rekall_common::Instant;

use super::*;

fn graph() -> deployment_graph::Model {
    deployment_graph::Model::new(Id::random(), "Release".into(), String::new(), Instant::now())
}

fn step(graph: &deployment_graph::Model, position: i32, title: &str) -> graph_step::Model {
    graph_step::Model {
        id: Id::random(),
        graph_id: graph.id,
        position,
        title: title.into(),
        body_markdown: None,
        connection_id: None,
        max_visits: None,
        created_at: Instant::now(),
    }
}

fn route(from: &graph_step::Model, to: Option<&graph_step::Model>, outcome: &str) -> graph_route::Model {
    graph_route::Model { id: Id::random(), graph_id: from.graph_id, from_step_id: from.id, to_step_id: to.map(|to| to.id), outcome: outcome.into(), position: 0 }
}

#[test]
fn a_loop_with_a_way_out_is_a_flow_that_can_run() {
    let graph = graph();
    let push = step(&graph, 0, "Push commit");
    let check = step(&graph, 1, "Check");
    let routes = vec![route(&push, Some(&check), "pushed"), route(&check, Some(&push), "error"), route(&check, None, "healthy")];
    let stored = StoredGraph { graph, steps: vec![push, check], routes };

    let plan = stored.plan().unwrap();
    assert_eq!(plan.steps[1].routes.iter().map(|route| route.to).collect::<Vec<_>>(), [Some(0), None]);
    assert_eq!(stored.problem().unwrap(), None);
}

#[test]
fn a_graph_without_routes_runs_as_a_checklist() {
    let graph = graph();
    let steps = vec![step(&graph, 0, "Push commit"), step(&graph, 1, "Deploy")];

    assert_eq!(StoredGraph { graph, steps, routes: Vec::new() }.problem().unwrap(), None);
}

#[test]
fn a_step_the_flow_never_reaches_is_why_it_cannot_run() {
    let graph = graph();
    let push = step(&graph, 0, "Push commit");
    let orphan = step(&graph, 1, "Orphan");
    let routes = vec![route(&push, None, "pushed")];
    let stored = StoredGraph { graph, steps: vec![push, orphan], routes };

    let problem = stored.problem().unwrap().expect("an unreachable step stops the graph from running");
    assert!(problem.contains("never reaches 'Orphan'"), "{problem}");
    assert_eq!(stored.response().unwrap().problem, Some(problem));
}
