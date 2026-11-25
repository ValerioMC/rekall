use rekall_core::step::TaskStepState;

use super::report;

#[path = "step_view_fixture.rs"]
mod step_view_fixture;
use step_view_fixture::step;

#[test]
fn the_answer_names_the_step_by_its_number_and_the_title_it_reads_now() {
    let rewritten = step(2, "Export as CSV", TaskStepState::Draft, Some("A download."));

    assert_eq!(
        report(&rewritten),
        "Step 3 rewritten, titled \"Export as CSV\". Its state, place, routes, skills and checks are as they were."
    );
}
