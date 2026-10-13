use rekall_common::{Id, Instant};
use rekall_model::step_link;
use rekall_model::step_visit;
use rekall_model::task_step;

use super::*;
use crate::step::StepFlowRows;

fn step(task_id: Id, title: &str, position: i32, state: TaskStepState) -> task_step::Model {
    let mut step = task_step::Model::new(task_id, title.into(), position, Instant::now());
    step.state = state;
    step
}

fn render(steps: &[task_step::Model], flow: &StepFlowRows) -> String {
    let views: Vec<TaskStepView> = steps.iter().map(|step| TaskStepView::of(step, flow).unwrap()).collect();
    ContextRenderer.render_steps(&views, None, &mut ComponentReferences::new(None))
}

#[test]
fn a_flow_names_its_current_step_its_moves_and_every_route() {
    let task_id = Id::random();
    let deploy = step(task_id, "Deploy", 0, TaskStepState::Done);
    let test = step(task_id, "Test", 1, TaskStepState::Open);
    let fix = step(task_id, "Fix the \"app\"", 2, TaskStepState::Open);
    let now = Instant::now();
    let mut left = step_visit::Model::enter(task_id, deploy.id, 1, now);
    left.leave(Some("deployed".into()), now);
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
