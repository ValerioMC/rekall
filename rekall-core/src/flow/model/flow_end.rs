use std::collections::HashSet;

use rekall_kernel::Id;
use rekall_store::{step_link, step_visit, task_step};

/// A checklist is a flow once a step a person released carries a route.
pub fn carries_routes(links: &[step_link::Model], released_steps: &[task_step::Model]) -> bool {
    let released: HashSet<Id> = released_steps.iter().map(|step| step.id).collect();
    links.iter().any(|link| released.contains(&link.from_step_id))
}

/// A flow has ended when it has run, stands on no step and leaves nothing it entered undone.
/// Steps it never entered stay open: the route taken did not need them. A visited step that is
/// open again was sent back, so the flow is live.
pub fn flow_has_ended(released_steps: &[task_step::Model], visits: &[step_visit::Model]) -> bool {
    let entered: HashSet<Id> = visits.iter().map(|visit| visit.step_id).collect();
    !visits.is_empty()
        && visits.iter().all(|visit| !visit.is_open())
        && released_steps.iter().all(|step| !step.state.running())
        && released_steps.iter().all(|step| !(step.state.unstarted() && entered.contains(&step.id)))
}

/// The steps a flow went through, in checklist order.
pub fn steps_entered<'a>(released_steps: &'a [task_step::Model], visits: &[step_visit::Model]) -> Vec<&'a task_step::Model> {
    let entered: HashSet<Id> = visits.iter().map(|visit| visit.step_id).collect();
    released_steps.iter().filter(|step| entered.contains(&step.id)).collect()
}

#[cfg(test)]
#[path = "../../../tests/unit/flow/model/flow_end_tests.rs"]
mod tests;
