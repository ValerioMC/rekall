use rekall_common::{Id, Instant};
use rekall_store::step_link;
use rekall_store::step_visit;
use rekall_store::task_step;

use super::*;
use crate::step::StepFlowRows;

fn step(task_id: Id, title: &str, position: i32, state: TaskStepState) -> task_step::Model {
    let mut step = task_step::Model::new(task_id, title.into(), position, Instant::now());
    step.state = state;
    step
}

fn render(steps: &[task_step::Model], flow: &StepFlowRows) -> String {
    let views: Vec<TaskStepView> = steps.iter().map(|step| TaskStepView::of(step, flow).unwrap()).collect();
    ContextRenderer.render_steps(&views, &[], None, &mut ComponentReferences::new(None))
}

#[test]
fn a_flow_names_its_current_step_its_moves_and_every_route() {
    let task_id = Id::random();
    let deploy = step(task_id, "Deploy", 0, TaskStepState::Done);
    let test = step(task_id, "Test", 1, TaskStepState::Open);
    let fix = step(task_id, "Fix the \"app\"", 2, TaskStepState::Open);
    let now = Instant::now();
    let mut left = step_visit::Model::enter(task_id, deploy.id, 1, now);
    left.leave(Some("deployed".into()), None, now);
    let flow = StepFlowRows {
        links: vec![
            step_link::Model::new(task_id, deploy.id, Some(test.id), "deployed".into(), 0, now),
            step_link::Model::new(task_id, test.id, Some(fix.id), "app-error".into(), 0, now),
            step_link::Model::new(task_id, fix.id, Some(deploy.id), "fixed".into(), 0, now),
        ],
        visits: vec![left, step_visit::Model::enter(task_id, test.id, 2, now)],
    };

    let rendered = render(&[deploy, test, fix], &flow);

    assert!(rendered.contains("flow=\"true\" current=\"2\" moves=\"2\""), "{rendered}");
    assert!(rendered.contains("- [ ] Test  (current, visit 1)"), "{rendered}");
    assert!(rendered.contains("  Routes: `app-error` -> step 3 \"Fix the 'app'\""), "{rendered}");
    assert!(rendered.contains("- [x] Deploy\n  Routes: `deployed` -> step 2 \"Test\""), "a finished step keeps its routes: {rendered}");
}

#[test]
fn a_plain_checklist_carries_no_flow_attributes_or_routes() {
    let task_id = Id::random();
    let rendered = render(&[step(task_id, "Only", 0, TaskStepState::Open)], &StepFlowRows::default());

    assert!(!rendered.contains("flow="), "{rendered}");
    assert!(!rendered.contains("Routes:"), "{rendered}");
}

#[test]
fn a_route_to_the_end_of_the_flow_is_named_as_such() {
    let task_id = Id::random();
    let test = step(task_id, "Test", 0, TaskStepState::Open);
    let fix = step(task_id, "Fix", 1, TaskStepState::Open);
    let now = Instant::now();
    let flow = StepFlowRows {
        links: vec![
            step_link::Model::new(task_id, test.id, None, "passed".into(), 0, now),
            step_link::Model::new(task_id, test.id, Some(fix.id), "app-error".into(), 1, now),
        ],
        visits: vec![],
    };

    let rendered = render(&[test, fix], &flow);

    assert!(rendered.contains("Routes: `passed` -> end of the flow; `app-error` -> step 2 \"Fix\""), "{rendered}");
}

#[test]
fn a_budget_shows_against_the_visits_on_the_current_step_and_the_others() {
    let task_id = Id::random();
    let mut deploy = step(task_id, "Deploy", 0, TaskStepState::Open);
    deploy.max_visits = Some(3);
    let mut test = step(task_id, "Test", 1, TaskStepState::Open);
    test.max_visits = Some(2);
    let now = Instant::now();
    let flow = StepFlowRows {
        links: vec![step_link::Model::new(task_id, deploy.id, Some(test.id), "deployed".into(), 0, now)],
        visits: vec![step_visit::Model::enter(task_id, deploy.id, 1, now)],
    };

    let rendered = render(&[deploy, test], &flow);

    assert!(rendered.contains("- [ ] Deploy  (current, visit 1 of 3)"), "{rendered}");
    assert!(rendered.contains("- [ ] Test  (entered 0 of at most 2 times)"), "{rendered}");
}

fn render_reaching(step: &task_step::Model, connection: StepConnectionView) -> String {
    let view = TaskStepView::of(step, &StepFlowRows::default()).unwrap();
    ContextRenderer.render_steps(&[view], &[connection], None, &mut ComponentReferences::new(None))
}

#[test]
fn an_open_step_names_the_connection_it_reaches() {
    let deploy = step(Id::random(), "Deploy", 0, TaskStepState::Open);
    let connection = StepConnectionView { step_id: deploy.id, label: "kmaster2".into(), bound_as: vec!["kmaster2".into()] };

    let rendered = render_reaching(&deploy, connection);

    assert!(rendered.contains("- [ ] Deploy\n  Connection: `kmaster2`, reached as its line in `<connections>` says."), "{rendered}");
}

#[test]
fn a_connection_the_project_binds_twice_names_both_bindings() {
    let deploy = step(Id::random(), "Deploy", 0, TaskStepState::Open);
    let connection = StepConnectionView { step_id: deploy.id, label: "gitlab".into(), bound_as: vec!["billing".into(), "orders".into()] };

    let rendered = render_reaching(&deploy, connection);

    assert!(rendered.contains("  Connection: `billing` or `orders`, reached as its line in `<connections>` says."), "{rendered}");
}

#[test]
fn a_connection_the_project_no_longer_binds_is_to_be_reported() {
    let deploy = step(Id::random(), "Deploy", 0, TaskStepState::Open);
    let connection = StepConnectionView { step_id: deploy.id, label: "kmaster2".into(), bound_as: Vec::new() };

    let rendered = render_reaching(&deploy, connection);

    assert!(rendered.contains("no longer uses"), "{rendered}");
    assert!(rendered.contains("Report it"), "{rendered}");
}

#[test]
fn a_finished_step_stays_its_title_alone_with_no_connection() {
    let deploy = step(Id::random(), "Deploy", 0, TaskStepState::Done);
    let connection = StepConnectionView { step_id: deploy.id, label: "kmaster2".into(), bound_as: vec!["kmaster2".into()] };

    let rendered = render_reaching(&deploy, connection);

    assert!(!rendered.contains("Connection:"), "{rendered}");
}
