//! Steps as a flow against a real database: routes set from the console, a session moving
//! through them, the visits that record where it went, and a flow a session proposes as drafts.

mod support;

use rekall_model::{task, task_step, TaskStepState};
use rekall_service::flow::{FlowPlan, FlowRoute, PlannedRoute, PlannedStep, RouteTarget};
use rekall_service::step::{FlowPlace, VisitBudget};
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
    links.add(build.id, RouteTarget::Step(deploy.id), Some("built")).await.unwrap();
    links.add(deploy.id, RouteTarget::Step(test.id), Some("deployed")).await.unwrap();
    links.add(deploy.id, RouteTarget::Step(fix_pipeline.id), Some("pipeline-error")).await.unwrap();
    links.add(test.id, RouteTarget::Step(fix_app.id), Some("app-error")).await.unwrap();
    links.add(fix_app.id, RouteTarget::Step(deploy.id), Some("fixed")).await.unwrap();
    links.add(fix_pipeline.id, RouteTarget::Step(deploy.id), Some("fixed")).await.unwrap();
    Pipeline { task, build, deploy, test, fix_app, fix_pipeline }
}

async fn claim(world: &World, step: &str, outcome: Option<&str>) -> rekall_service::flow::FlowMove {
    world.services.flow.transition(Some("vega"), "release", Some(step), TaskStepState::Claimed, outcome, None).await.unwrap()
}

async fn start(world: &World, step: &str) {
    world.services.flow.transition(Some("vega"), "release", Some(step), TaskStepState::Running, None, None).await.unwrap();
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
        world.services.flow.transition(Some("vega"), "release", Some("Deploy"), TaskStepState::Claimed, None, None).await;

    assert!(refused.unwrap_err().to_string().contains("`deployed`, `pipeline-error`"));
    let all = views(&world, &pipeline).await;
    assert_eq!(view(&all, &pipeline.deploy).state, TaskStepState::Open, "the claim rolled back with the refusal");
}

#[tokio::test]
async fn a_step_with_no_routes_out_ends_the_flow() {
    let world = world().await;
    let pipeline = pipeline(&world).await;
    world.services.step_links.add(pipeline.test.id, RouteTarget::Step(pipeline.build.id), Some("passed")).await.unwrap();
    let ship = world.step(&pipeline.task, "Ship", 5, TaskStepState::Open).await;
    let passed = views(&world, &pipeline).await;
    let route_id = view(&passed, &pipeline.test).links.iter().find(|link| link.outcome == "passed").unwrap().id;
    world.services.step_links.edit(route_id, Some(RouteTarget::Step(ship.id)), None).await.unwrap();

    claim(&world, "Test", Some("passed")).await;
    let ended = claim(&world, "Ship", None).await;

    assert!(matches!(ended.route, Some(FlowRoute::Ended { .. })));
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
async fn a_claim_keeps_its_report_on_the_visit_it_leaves() {
    let world = world().await;
    let pipeline = pipeline(&world).await;
    claim(&world, "Build", None).await;
    claim(&world, "Deploy", Some("deployed")).await;

    world.services.flow.transition(Some("vega"), "release", Some("Test"), TaskStepState::Claimed, Some("app-error"), Some("  `GET /health` answers 500.  ")).await.unwrap();

    let all = views(&world, &pipeline).await;
    let left = &view(&all, &pipeline.test).visits[0];
    assert_eq!(left.outcome.as_deref(), Some("app-error"));
    assert_eq!(left.report.as_deref(), Some("`GET /health` answers 500."), "kept stripped");
}

#[tokio::test]
async fn a_report_is_refused_off_a_claim_on_a_flow_or_past_its_length() {
    let world = world().await;
    pipeline(&world).await;
    let flow = &world.services.flow;

    let on_a_start = flow.transition(Some("vega"), "release", Some("Build"), TaskStepState::Running, None, Some("early")).await;
    let too_long = flow.transition(Some("vega"), "release", Some("Build"), TaskStepState::Claimed, None, Some(&"x".repeat(4_001))).await;

    assert!(on_a_start.is_err(), "a report belongs to the claim that leaves a step");
    assert!(too_long.unwrap_err().to_string().contains("keep it to 4000"));
}

#[tokio::test]
async fn a_report_on_a_plain_checklist_is_refused() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "release").await;
    world.step(&task, "Only", 0, TaskStepState::Open).await;

    let refused = world.services.flow.transition(Some("vega"), "release", Some("1"), TaskStepState::Claimed, None, Some("it broke")).await;

    assert!(refused.is_err());
}

#[tokio::test]
async fn an_outcome_on_a_plain_checklist_is_refused() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "release").await;
    world.step(&task, "Only", 0, TaskStepState::Open).await;

    let refused = world.services.flow.transition(Some("vega"), "release", Some("1"), TaskStepState::Claimed, Some("passed"), None).await;

    assert!(refused.is_err());
}

#[tokio::test]
async fn a_plain_checklist_moves_without_recording_visits() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "release").await;
    world.step(&task, "Only", 0, TaskStepState::Open).await;

    let moved = world.services.flow.transition(Some("vega"), "release", Some("1"), TaskStepState::Claimed, None, None).await.unwrap();

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

    assert!(links.add(pipeline.build.id, RouteTarget::Step(stranger.id), Some("x")).await.is_err(), "across tasks");
    assert!(links.add(pipeline.build.id, RouteTarget::Step(pipeline.build.id), Some("again")).await.is_err(), "onto itself");
    let taken = links.add(pipeline.deploy.id, RouteTarget::Step(pipeline.build.id), Some("Deployed")).await.unwrap_err();
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

    let refused = world.services.flow.transition(Some("vega"), "release", Some("Build"), TaskStepState::Claimed, None, None).await;

    assert!(refused.unwrap_err().to_string().contains("still a draft"));
}

#[tokio::test]
async fn a_route_to_the_end_finishes_the_flow_with_its_outcome() {
    let world = world().await;
    let pipeline = pipeline(&world).await;
    world.services.step_links.add(pipeline.test.id, RouteTarget::End, Some("passed")).await.unwrap();

    claim(&world, "Test", Some("passed")).await;
    let finished = claim(&world, "Test", Some("passed")).await;

    let Some(FlowRoute::Ended { outcome }) = finished.route else { panic!("expected the flow to end") };
    assert_eq!(outcome.as_deref(), Some("passed"));
    let all = views(&world, &pipeline).await;
    assert!(all.iter().all(|step| !step.is_current()));
    let route = view(&all, &pipeline.test).links.iter().find(|link| link.outcome == "passed").unwrap();
    assert_eq!(route.to_step_id, None);
}

#[tokio::test]
async fn a_dragged_step_keeps_its_place_until_the_layout_is_cleared() {
    let world = world().await;
    let pipeline = pipeline(&world).await;

    let placed = world.services.steps.place_in_flow(pipeline.test.id, Some(FlowPlace::new(120, -40).unwrap())).await.unwrap();
    assert_eq!((view(&placed, &pipeline.test).flow_x, view(&placed, &pipeline.test).flow_y), (Some(120), Some(-40)));

    let cleared = world.services.steps.clear_flow_places(pipeline.task.id).await.unwrap();
    assert_eq!((view(&cleared, &pipeline.test).flow_x, view(&cleared, &pipeline.test).flow_y), (None, None));
}

// --- Visit budgets

async fn budget(world: &World, step: &task_step::Model, visits: i32) {
    world.services.steps.limit_visits(step.id, Some(VisitBudget::new(visits).unwrap())).await.unwrap();
}

/// Goes round deploy, test, fix once: deploy is entered twice.
async fn go_round_once(world: &World) {
    claim(world, "Build", None).await;
    claim(world, "Deploy", Some("deployed")).await;
    claim(world, "Test", Some("app-error")).await;
    claim(world, "Fix the application", None).await;
}

#[tokio::test]
async fn a_route_into_a_step_whose_budget_is_spent_is_refused_and_the_claim_rolls_back() {
    let world = world().await;
    let pipeline = pipeline(&world).await;
    budget(&world, &pipeline.deploy, 2).await;
    go_round_once(&world).await;
    claim(&world, "Deploy", Some("deployed")).await;
    claim(&world, "Test", Some("app-error")).await;

    let refused =
        world.services.flow.transition(Some("vega"), "release", Some("Fix the application"), TaskStepState::Claimed, None, None).await;

    assert!(refused.unwrap_err().to_string().contains("entered 2 times, its whole visit budget"));
    let all = views(&world, &pipeline).await;
    assert!(view(&all, &pipeline.fix_app).is_current(), "the fix keeps its open visit");
    assert_ne!(view(&all, &pipeline.fix_app).state, TaskStepState::Claimed, "the claim was rolled back");
    assert_eq!(view(&all, &pipeline.deploy).visits.len(), 2);
}

#[tokio::test]
async fn standing_on_a_step_does_not_spend_its_budget_and_a_reset_renews_it() {
    let world = world().await;
    let pipeline = pipeline(&world).await;
    budget(&world, &pipeline.deploy, 1).await;
    start(&world, "Deploy").await;
    start(&world, "Deploy").await;
    claim(&world, "Deploy", Some("deployed")).await;
    claim(&world, "Test", Some("app-error")).await;
    assert!(world.services.flow.transition(Some("vega"), "release", Some("Fix the application"), TaskStepState::Claimed, None, None).await.is_err());

    world.services.flow.reset(pipeline.task.id).await.unwrap();
    let back = claim(&world, "Fix the application", None).await;

    let Some(FlowRoute::Followed { next, .. }) = back.route else { panic!("expected the route back to deploy") };
    assert_eq!(next.id, pipeline.deploy.id);
}

#[tokio::test]
async fn lifting_the_budget_lets_the_flow_go_round_again() {
    let world = world().await;
    let pipeline = pipeline(&world).await;
    budget(&world, &pipeline.deploy, 1).await;
    claim(&world, "Build", None).await;
    claim(&world, "Deploy", Some("deployed")).await;
    claim(&world, "Test", Some("app-error")).await;

    world.services.steps.limit_visits(pipeline.deploy.id, None).await.unwrap();

    assert!(claim(&world, "Fix the application", None).await.route.is_some());
    assert_eq!(view(&views(&world, &pipeline).await, &pipeline.deploy).max_visits, None);
}

// --- A flow a session proposes

fn planned(title: &str, routes: &[(&str, Option<usize>)]) -> PlannedStep {
    PlannedStep {
        title: title.into(),
        detail: Some(format!("What {title} does.")),
        routes: routes.iter().map(|(outcome, to)| PlannedRoute { outcome: (*outcome).into(), to: *to }).collect(),
        max_visits: None,
    }
}

/// Deploy, test; a failed test goes to a fix that goes back to deploy, a pass ends the flow.
fn deploy_test_fix() -> FlowPlan {
    FlowPlan::new(vec![
        planned("Deploy", &[("deployed", Some(1))]),
        planned("Test", &[("passed", None), ("app-error", Some(2))]),
        PlannedStep { max_visits: Some(VisitBudget::new(3).unwrap()), ..planned("Fix", &[("fixed", Some(0))]) },
    ])
}

async fn promote_all(world: &World, task: &task::Model) {
    for step in world.services.steps.find_by_task(task.id).await.unwrap() {
        world.services.steps.edit(step.id, None, None, None, Some(false)).await.unwrap();
    }
}

#[tokio::test]
async fn a_proposed_flow_lands_as_drafts_joined_by_their_routes_and_runs_once_promoted() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "release").await;

    let proposed = world.services.flow_proposals.propose(Some("vega"), "release", &deploy_test_fix()).await.unwrap();

    assert_eq!((proposed.steps, proposed.routes), (3, 4));
    let drafts = world.services.steps.find_by_task(task.id).await.unwrap();
    assert!(drafts.iter().all(|step| step.state == TaskStepState::Draft));
    let fix = drafts.iter().find(|step| step.title == "Fix").unwrap();
    assert_eq!(fix.links[0].to_step_id, Some(drafts[0].id), "the fix routes back to deploy");
    assert_eq!(fix.body_markdown.as_deref(), Some("What Fix does."));
    assert_eq!(fix.max_visits, Some(3));

    promote_all(&world, &task).await;
    world.services.flow.transition(Some("vega"), "release", Some("Deploy"), TaskStepState::Claimed, None, None).await.unwrap();
    let failed =
        world.services.flow.transition(Some("vega"), "release", Some("Test"), TaskStepState::Claimed, Some("app-error"), None).await.unwrap();
    let Some(FlowRoute::Followed { next, .. }) = failed.route else { panic!("expected the app-error route") };
    assert_eq!(next.title, "Fix");
}

#[tokio::test]
async fn routes_between_drafts_do_not_make_the_checklist_a_flow_until_they_are_promoted() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "release").await;
    world.services.flow_proposals.propose(Some("vega"), "release", &deploy_test_fix()).await.unwrap();
    world.step(&task, "Write the notes", 3, TaskStepState::Open).await;

    let refused =
        world.services.flow.transition(Some("vega"), "release", Some("Write the notes"), TaskStepState::Claimed, Some("done"), None).await;

    assert!(refused.unwrap_err().to_string().contains("carry no routes"));
}

#[tokio::test]
async fn a_flow_is_not_proposed_onto_a_task_that_has_steps_and_nothing_is_written() {
    let world = world().await;
    let project = world.project("vega", None, false).await;
    let task = world.task(&project, "release").await;
    world.step(&task, "Build", 0, TaskStepState::Open).await;

    let refused = world.services.flow_proposals.propose(Some("vega"), "release", &deploy_test_fix()).await;

    assert!(refused.unwrap_err().to_string().contains("onto a task with no steps"));
    assert_eq!(world.services.steps.find_by_task(task.id).await.unwrap().len(), 1);
}

#[tokio::test]
async fn a_session_proposes_routes_between_drafts_only() {
    let world = world().await;
    let pipeline = pipeline(&world).await;
    let draft = world.step(&pipeline.task, "Draft", 5, TaskStepState::Draft).await;
    let tx = rekall_service::Tx::write(&world.services.ctx).await.unwrap();

    let from_released = world.services.step_links.propose_in(&tx, pipeline.test.id, RouteTarget::Step(draft.id), "next").await;
    let into_released = world.services.step_links.propose_in(&tx, draft.id, RouteTarget::Step(pipeline.test.id), "next").await;

    assert!(from_released.unwrap_err().to_string().contains("between drafts only"));
    assert!(into_released.unwrap_err().to_string().contains("between drafts only"));
    world.services.step_links.propose_in(&tx, draft.id, RouteTarget::End, "done").await.unwrap();
    tx.rollback().await.unwrap();
}

#[test]
fn a_place_far_from_the_origin_is_refused() {
    assert!(FlowPlace::new(1_000_000, 0).is_err());
}
