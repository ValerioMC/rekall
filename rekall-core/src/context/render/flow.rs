//! A flow's attributes on the checklist tag and the routes each step lists.

use crate::step::{flow_entry, StepLinkView, TaskStepView};

use super::text::attribute;

pub(super) fn is_flow(steps: &[&TaskStepView]) -> bool {
    steps.iter().any(|step| !step.links.is_empty())
}

/// `flow`, the step the work stands on (one-based, as `rekall_step` takes it) and how many moves it took.
pub(super) fn flow_attributes(steps: &[&TaskStepView]) -> String {
    let moves: usize = steps.iter().map(|step| step.visits.len()).sum();
    let mut attributes = String::from(" flow=\"true\"");
    if let Some(current) = current_step_of(steps, moves) {
        attributes.push_str(&format!(" current=\"{}\"", current.position + 1));
    }
    attributes.push_str(&format!(" moves=\"{moves}\""));
    attributes
}

/// The step the flow stands on. A flow laid after finished steps and not yet entered stands on its
/// entry, which is where the work goes next; a flow from the first step names none until it starts.
fn current_step_of<'a>(steps: &[&'a TaskStepView], moves: usize) -> Option<&'a TaskStepView> {
    if let Some(standing) = steps.iter().find(|step| step.is_current()) {
        return Some(standing);
    }
    let entry = flow_entry(steps.iter().copied())?;
    (moves == 0 && entry.position > 0).then_some(entry)
}

/// A step's routes, each naming its target by number and title. Finished steps get them too: a
/// route can bring the work back.
pub(super) fn render_routes(step: &TaskStepView, steps: &[&TaskStepView]) -> String {
    if step.links.is_empty() {
        return String::new();
    }
    let routes: Vec<String> = step.links.iter().filter_map(|link| render_route(link, steps)).collect();
    if routes.is_empty() {
        return String::new();
    }
    format!("  Routes: {}\n", routes.join("; "))
}

/// One route: `outcome` -> a step by number and title, or the end of the flow. A route to a step
/// that is gone is left out.
fn render_route(link: &StepLinkView, steps: &[&TaskStepView]) -> Option<String> {
    let Some(to_step_id) = link.to_step_id else {
        return Some(format!("`{}` -> end of the flow", link.outcome));
    };
    let target = steps.iter().find(|candidate| candidate.id == to_step_id)?;
    Some(format!("`{}` -> step {} \"{}\"", link.outcome, target.position + 1, attribute(&target.title)))
}
