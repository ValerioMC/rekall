//! The checklist: each step with its state, its detail while it is open, what a flow reported as it
//! arrived, and its routes.

use rekall_kernel::jstr;
use rekall_store::TaskStepState;

use crate::component::ComponentReferences;
use crate::context::StepConnectionView;
use crate::step::TaskStepView;
use crate::wrapup::WrapupView;

use super::super::model::context_renderer::ContextRenderer;
use super::flow::{flow_attributes, is_flow, render_routes};
use super::text::{indent, truncate, MAX_DOCUMENT_CHARACTERS};

impl ContextRenderer {
    pub fn render_steps(
        &self,
        all_steps: &[TaskStepView],
        connections: &[StepConnectionView],
        wrapup: Option<&WrapupView>,
        references: &mut ComponentReferences,
    ) -> String {
        let drafts = all_steps.iter().filter(|s| s.state.draft()).count();
        let steps: Vec<&TaskStepView> = all_steps.iter().filter(|s| !s.state.draft()).collect();
        if steps.is_empty() {
            return String::new();
        }
        let mut out = format!("\n<steps {}>\n", steps_attributes(&steps, drafts, wrapup));
        for (index, step) in steps.iter().enumerate() {
            let connection = connections.iter().find(|connection| connection.step_id == step.id);
            out.push_str(&render_step(step, connection, wrapup, references, index > 0 && step.starts_session));
            out.push_str(&render_arrival(step, &steps));
            out.push_str(&render_routes(step, &steps));
        }
        if drafts > 0 {
            out.push_str(&format!(
                "<!-- {drafts} further step{} still in draft, not shown: promoted to the checklist \
                 in the console when it is ready to work -->\n",
                if drafts == 1 { "" } else { "s" }
            ));
        }
        out.push_str("</steps>\n");
        out
    }
}

/// The checklist's counts; one that is zero is left out, except done and open.
fn steps_attributes(steps: &[&TaskStepView], drafts: usize, wrapup: Option<&WrapupView>) -> String {
    let done = steps.iter().filter(|s| s.state.complete()).count();
    let running = steps.iter().filter(|s| s.state.running()).count();
    let awaiting = steps.iter().filter(|s| s.state == TaskStepState::Claimed).count();
    let unwritten = steps.iter().filter(|s| is_unwritten(s, wrapup)).count();
    let mut attributes = format!("done=\"{done}\" open=\"{}\"", steps.len() - done);
    if is_flow(steps) {
        attributes.push_str(&flow_attributes(steps));
    }
    let sessions = 1 + steps.iter().skip(1).filter(|s| s.starts_session).count();
    if sessions > 1 {
        attributes.push_str(&format!(" sessions=\"{sessions}\""));
    }
    let optional = [("running", running), ("awaiting-review", awaiting), ("finished-since-wrapup", unwritten), ("draft", drafts)];
    for (name, count) in optional {
        if count > 0 {
            attributes.push_str(&format!(" {name}=\"{count}\""));
        }
    }
    attributes
}

/// A closed step is its title alone, unless the wrapup predates it: then, like an open one, it carries its detail.
fn render_step(
    step: &TaskStepView,
    connection: Option<&StepConnectionView>,
    wrapup: Option<&WrapupView>,
    references: &mut ComponentReferences,
    opens_session: bool,
) -> String {
    let unwritten = is_unwritten(step, wrapup);
    let mut out = String::from(if step.state.complete() { "- [x] " } else { "- [ ] " });
    out.push_str(&step.title);
    if step.is_current() {
        out.push_str(&format!("  (current, {})", step.visit_label()));
    } else if let Some(max) = step.max_visits {
        out.push_str(&format!("  (entered {} of at most {max} times)", step.visits.len()));
    }
    if step.state.running() {
        out.push_str("  (in progress)");
    } else if step.state == TaskStepState::Claimed {
        out.push_str("  (claimed, waiting for the console to accept it)");
    }
    if unwritten {
        out.push_str("  (finished since the wrapup was written)");
    }
    if opens_session {
        out.push_str("  (opens a new session)");
    }
    out.push('\n');
    if step.state.complete() && !unwritten {
        return out;
    }
    if let Some(connection) = connection {
        out.push_str(&render_step_connection(connection));
    }
    if !step.passes.is_empty() {
        out.push_str(&indent(&render_passes(step, references)));
        out.push('\n');
    } else if let Some(body) = step.body_markdown.as_deref().filter(|body| !jstr::is_blank(body)) {
        out.push_str(&indent(&truncate(Some(&references.expand(body)), MAX_DOCUMENT_CHARACTERS)));
        out.push('\n');
    }
    out
}

/// On the current step, what the step the flow came from reported as it left, so a pass sent here
/// to fix something works from what was seen rather than from a session's memory.
fn render_arrival(step: &TaskStepView, steps: &[&TaskStepView]) -> String {
    let Some(open) = step.visits.iter().find(|visit| visit.is_open()) else {
        return String::new();
    };
    let Some((from, left)) = steps
        .iter()
        .find_map(|candidate| candidate.visits.iter().find(|visit| visit.ordinal == open.ordinal - 1).map(|visit| (candidate, visit)))
    else {
        return String::new();
    };
    let Some(report) = left.report.as_deref() else {
        return String::new();
    };
    let outcome = left.outcome.as_deref().unwrap_or("its only route");
    let arrival = format!(
        "The flow came here from step {number} \"{}\", which ended `{outcome}` and reported this. It is what this pass works from:\n\n\
         <arrived-from step=\"{number}\" outcome=\"{outcome}\">\n{}\n</arrived-from>",
        from.title,
        truncate(Some(report), MAX_DOCUMENT_CHARACTERS),
        number = from.position + 1
    );
    format!("{}\n", indent(&arrival))
}

/// The connection a step works through. One the project no longer binds has no line in
/// `<connections>`, so the session is told to report it rather than reach it some other way.
fn render_step_connection(connection: &StepConnectionView) -> String {
    if !connection.bound_as.is_empty() {
        let labels = connection.bound_as.iter().map(|label| format!("`{label}`")).collect::<Vec<_>>().join(" or ");
        return format!("  Connection: {labels}, reached as its line in `<connections>` says.\n");
    }
    format!(
        "  Connection: `{}`, which this project no longer uses, so `<connections>` does not list it. Report it; \
         do not reach the system another way.\n",
        connection.label
    )
}

/// A step sent back after review: what each earlier pass worked from, then the feedback this
/// pass acts on, so a session builds on its work instead of reading the feedback as a new brief.
fn render_passes(step: &TaskStepView, references: &mut ComponentReferences) -> String {
    let mut out = String::from("Sent back after review. Previously you have executed this:\n");
    for (at, pass) in step.passes.iter().enumerate() {
        out.push_str(&format!("\n<previous-pass number=\"{}\" sent-back=\"{}\">\n", at + 1, pass.sent_back_at));
        match pass.detail_markdown.as_deref().filter(|body| !jstr::is_blank(body)) {
            Some(body) => out.push_str(&truncate(Some(&references.expand(body)), MAX_DOCUMENT_CHARACTERS)),
            None => out.push_str("(no detail beyond the title)"),
        }
        out.push_str("\n</previous-pass>\n");
    }
    out.push_str("\nBut the user's feedback is this, and it is what this pass has to do:\n\n<feedback>\n");
    match step.body_markdown.as_deref().filter(|body| !jstr::is_blank(body)) {
        Some(body) => out.push_str(&truncate(Some(&references.expand(body)), MAX_DOCUMENT_CHARACTERS)),
        None => out.push_str("(not written yet: ask the user what the pass should change)"),
    }
    out.push_str("\n</feedback>");
    out
}

fn is_unwritten(step: &TaskStepView, wrapup: Option<&WrapupView>) -> bool {
    if !step.state.complete() {
        return false;
    }
    let Some(completed) = step.completed_at() else {
        return false;
    };
    match wrapup {
        None => true,
        Some(wrapup) => completed.is_after(&wrapup.updated_at),
    }
}
