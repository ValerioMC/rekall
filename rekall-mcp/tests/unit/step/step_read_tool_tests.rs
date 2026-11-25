use rekall_core::step::{StepLinkView, TaskStepState, TaskStepView, UnstartedStepsView};
use rekall_kernel::Id;

use super::report;
use crate::step::step_view_fixture::step;

fn route(from: &TaskStepView, outcome: &str, to: Option<&TaskStepView>) -> StepLinkView {
    StepLinkView { id: Id::random(), from_step_id: from.id, to_step_id: to.map(|step| step.id), outcome: outcome.into(), position: 0 }
}

#[test]
fn each_step_still_to_do_comes_with_its_number_state_and_detail_as_stored() {
    let built = step(0, "Built", TaskStepState::Done, Some("finished"));
    let wire = step(1, "Wire", TaskStepState::Open, Some("Reuse [[storage]]."));
    let docs = step(2, "Docs", TaskStepState::Draft, None);

    let read = report(&UnstartedStepsView::of("task:VG-1".into(), vec![built, wire, docs]));

    assert_eq!(read, "Draft and open steps of `task:VG-1`, as stored:\n\n## Step 2 (open): Wire\n<detail>\nReuse [[storage]].\n</detail>\n\n## Step 3 (draft): Docs\nNo detail.");
}

#[test]
fn a_route_names_the_number_of_its_step_even_a_finished_one_or_the_end_of_the_flow() {
    let test = step(0, "Test", TaskStepState::Done, None);
    let mut fix = step(1, "Fix", TaskStepState::Open, None);
    fix.links = vec![route(&fix, "again", Some(&test)), route(&fix, "fixed", None)];

    let read = report(&UnstartedStepsView::of("task:VG-1".into(), vec![test, fix]));

    assert!(read.contains("Routes: `again` -> step 1; `fixed` -> end of the flow"), "{read}");
}

#[test]
fn a_task_with_nothing_left_to_do_says_so() {
    let built = step(0, "Built", TaskStepState::Claimed, None);

    assert_eq!(report(&UnstartedStepsView::of("task:VG-1".into(), vec![built])), "`task:VG-1` has no draft or open step.");
}
