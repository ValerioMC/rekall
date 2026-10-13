//! Steps as a flow against a real database: routes set from the console, a session moving
//! through them, and the visits that record where it went.

mod support;

use rekall_model::{task, task_step, TaskStepState};
use rekall_service::flow::FlowRoute;
use rekall_service::step::TaskStepView;
use support::{world, World};

/// Build, deploy, test; a failed test routes to a fix, which routes back to deploy.
struct Pipeline {
    task: task::Model,
    build: task_step::Model,
    deploy: task_step::Model,
    test: task_step::Model,
    fix_app: task_step::Model,
    fix_pipeline: task_step::Model,
}

async fn pipeline(world: &World) -> Pipeline {
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "release").await;
    let build = world.step(&task, "Build", 0, TaskStepState::Open).await;
    let deploy = world.step(&task, "Deploy", 1, TaskStepState::Open).await;
    let test = world.step(&task, "Test", 2, TaskStepState::Open).await;
    let fix_app = world.step(&task, "Fix the application", 3, TaskStepState::Open).await;
    let fix_pipeline = world.step(&task, "Fix the pipeline", 4, TaskStepState::Open).await;
    let links = &world.services.step_links;
    links.add(build.id, deploy.id, Some("built")).await.unwrap();
    links.add(deploy.id, test.id, Some("deployed")).await.unwrap();
    links.add(deploy.id, fix_pipeline.id, Some("pipeline-error")).await.unwrap();
    links.add(test.id, fix_app.id, Some("app-error")).await.unwrap();
    links.add(fix_app.id, deploy.id, Some("fixed")).await.unwrap();
    links.add(fix_pipeline.id, deploy.id, Some("fixed")).await.unwrap();
    Pipeline { task, build, deploy, test, fix_app, fix_pipeline }
}

async fn claim(world: &World, step: &str, outcome: Option<&str>) -> rekall_service::flow::FlowMove {
    world.services.flow.transition(Some("vega"), "release", Some(step), TaskStepState::Claimed, outcome).await.unwrap()
}

async fn start(world: &World, step: &str) {
    world.services.flow.transition(Some("vega"), "release", Some(step), TaskStepState::Running, None).await.unwrap();
}

async fn views(world: &World, pipeline: &Pipeline) -> Vec<TaskStepView> {
    world.services.steps.find_by_task(pipeline.task.id).await.unwrap()
}

fn view<'a>(all: &'a [TaskStepView], step: &task_step::Model) -> &'a TaskStepView {
    all.iter().find(|candidate| candidate.id == step.id).unwrap()
}

#[tokio::test]
async fn a_claim_with_an_outcome_follows_its_route_and_enters_the_next_step() {
    let world = world().await;
    let pipeline = pipeline(&world).await;

    start(&world, "1").await;
    let moved = claim(&world, "1", None).await;

    let FlowRoute::Followed { outcome, next } = moved.route.unwrap() else { panic!("expected a route") };
    assert_eq!(outcome, "built");
    assert_eq!(next.id, pipeline.deploy.id);
    let all = views(&world, &pipeline).await;
    assert!(view(&all, &pipeline.deploy).is_current());
    assert_eq!(view(&all, &pipeline.build).visits[0].outcome.as_deref(), Some("built"));
}

#[tokio::test]
async fn a_loop_back_reopens_the_finished_step_and_counts_each_visit() {
    let world = world().await;
    let pipeline = pipeline(&world).await;

    claim(&world, "Build", None).await;
    claim(&world, "Deploy", Some("deployed")).await;
    claim(&world, "Test", Some("App error")).await;
    let back = claim(&world, "Fix the application", None).await;

    let FlowRoute::Followed { next, .. } = back.route.unwrap() else { panic!("expected a route") };
    assert_eq!(next.state, TaskStepState::Open, "the claimed deploy is open again for another pass");
    let all = views(&world, &pipeline).await;
    assert_eq!(view(&all, &pipeline.deploy).visits.len(), 2);
    assert_eq!(view(&all, &pipeline.test).visits[0].outcome.as_deref(), Some("app-error"));
    let ordinals: Vec<i32> = all.iter().flat_map(|step| step.visits.iter().map(|visit| visit.ordinal)).collect();
    let mut sorted = ordinals.clone();
    sorted.sort();
    assert_eq!(sorted, vec![1, 2, 3, 4, 5], "one move number per entry, dense");
}

#[tokio::test]
async fn a_step_with_several_routes_refuses_a_claim_without_an_outcome_and_moves_nothing() {
    let world = world().await;
    let pipeline = pipeline(&world).await;

    let refused =
        world.services.flow.transition(Some("vega"), "release", Some("Deploy"), TaskStepState::Claimed, None).await;

    assert!(refused.unwrap_err().to_string().contains("`deployed`, `pipeline-error`"));
    let all = views(&world, &pipeline).await;
    assert_eq!(view(&all, &pipeline.deploy).state, TaskStepState::Open, "the claim rolled back with the refusal");
}

#[tokio::test]
async fn a_step_with_no_routes_out_ends_the_flow() {
    let world = world().await;
    let pipeline = pipeline(&world).await;
    world.services.step_links.add(pipeline.test.id, pipeline.build.id, Some("passed")).await.unwrap();
    let ship = world.step(&pipeline.task, "Ship", 5, TaskStepState::Open).await;
    let passed = views(&world, &pipeline).await;
    let route_id = view(&passed, &pipeline.test).links.iter().find(|link| link.outcome == "passed").unwrap().id;
    world.services.step_links.edit(route_id, Some(ship.id), None).await.unwrap();

    claim(&world, "Test", Some("passed")).await;
    let ended = claim(&world, "Ship", None).await;

    assert!(matches!(ended.route, Some(FlowRoute::Ended)));
    assert!(views(&world, &pipeline).await.iter().all(|step| !step.is_current()));
}

#[tokio::test]
async fn starting_another_step_leaves_the_current_one_without_a_route() {
    let world = world().await;
    let pipeline = pipeline(&world).await;

    start(&world, "Build").await;
    start(&world, "Fix the pipeline").await;

    let all = views(&world, &pipeline).await;
    assert!(view(&all, &pipeline.fix_pipeline).is_current());
    let left = &view(&all, &pipeline.build).visits[0];
    assert!(left.left_at.is_some() && left.outcome.is_none());
}

#[tokio::test]
async fn an_outcome_on_a_plain_checklist_is_refused() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "release").await;
    world.step(&task, "Only", 0, TaskStepState::Open).await;

    let refused = world.services.flow.transition(Some("vega"), "release", Some("1"), TaskStepState::Claimed, Some("passed")).await;

    assert!(refused.is_err());
}

#[tokio::test]
async fn a_plain_checklist_moves_without_recording_visits() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "release").await;
    world.step(&task, "Only", 0, TaskStepState::Open).await;

    let moved = world.services.flow.transition(Some("vega"), "release", Some("1"), TaskStepState::Claimed, None).await.unwrap();

    assert!(moved.route.is_none());
    assert!(moved.step.visits.is_empty());
}

#[tokio::test]
async fn resetting_drops_every_visit_and_keeps_the_states() {
    let world = world().await;
    let pipeline = pipeline(&world).await;
    claim(&world, "Build", None).await;

    let after = world.services.flow.reset(pipeline.task.id).await.unwrap();

    assert!(after.iter().all(|step| step.visits.is_empty()));
    assert_eq!(view(&after, &pipeline.build).state, TaskStepState::Claimed);
}

#[tokio::test]
async fn a_route_needs_two_steps_of_one_task_and_a_free_outcome() {
    let world = world().await;
    let pipeline = pipeline(&world).await;
    let other_task = world.task(&world.project("beacon", None, false).await, "other").await;
    let stranger = world.step(&other_task, "Elsewhere", 0, TaskStepState::Open).await;
    let links = &world.services.step_links;

    assert!(links.add(pipeline.build.id, stranger.id, Some("x")).await.is_err(), "across tasks");
    assert!(links.add(pipeline.build.id, pipeline.build.id, Some("again")).await.is_err(), "onto itself");
    let taken = links.add(pipeline.deploy.id, pipeline.build.id, Some("Deployed")).await.unwrap_err();
    assert!(taken.to_string().contains("already routes by `deployed`"), "{taken}");
}

#[tokio::test]
async fn deleting_a_step_drops_the_routes_that_touch_it() {
    let world = world().await;
    let pipeline = pipeline(&world).await;

    world.services.steps.delete(pipeline.fix_app.id).await.unwrap();

    let all = views(&world, &pipeline).await;
    assert!(view(&all, &pipeline.test).links.is_empty());
}

#[tokio::test]
async fn a_route_into_a_draft_is_refused_when_the_flow_reaches_it() {
    let world = world().await;
    let pipeline = pipeline(&world).await;
    world.services.steps.edit(pipeline.deploy.id, None, None, None, Some(true)).await.unwrap();

    let refused = world.services.flow.transition(Some("vega"), "release", Some("Build"), TaskStepState::Claimed, None).await;

    assert!(refused.unwrap_err().to_string().contains("still a draft"));
}
