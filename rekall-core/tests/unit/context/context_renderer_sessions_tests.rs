use rekall_kernel::{Id, Instant};
use rekall_store::task_step;

use super::*;
use crate::step::StepFlowRows;

fn step(task_id: Id, title: &str, position: i32, starts_session: bool) -> task_step::Model {
    let mut step = task_step::Model::new(task_id, title.into(), position, Instant::now());
    step.state = TaskStepState::Open;
    step.starts_session = starts_session;
    step
}

fn render(steps: &[task_step::Model]) -> String {
    let views: Vec<TaskStepView> = steps.iter().map(|step| TaskStepView::of(step, &StepFlowRows::default()).unwrap()).collect();
    ContextRenderer.render_steps(&views, &[], None, &mut ComponentReferences::new(None))
}

#[test]
fn a_step_that_opens_a_session_says_so_and_the_checklist_counts_the_sessions() {
    let task_id = Id::random();

    let rendered = render(&[step(task_id, "Move", 0, true), step(task_id, "Rewire", 1, false), step(task_id, "Docs", 2, true)]);

    assert!(rendered.contains("sessions=\"2\""), "{rendered}");
    assert!(rendered.contains("- [ ] Docs  (opens a new session)"), "{rendered}");
    assert!(!rendered.contains("- [ ] Move  (opens"), "the first step opens the first session without saying so: {rendered}");
    assert!(!rendered.contains("- [ ] Rewire  (opens"), "{rendered}");
}

#[test]
fn a_checklist_with_no_session_marks_runs_in_one_session_and_says_nothing() {
    let task_id = Id::random();

    let rendered = render(&[step(task_id, "One", 0, false), step(task_id, "Two", 1, false)]);

    assert!(!rendered.contains("sessions="), "{rendered}");
    assert!(!rendered.contains("new session"), "{rendered}");
}
