use rekall_store::{FeedbackKind, TaskStepState};

/// The pending revision a description edit keeps rewriting.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenRevision<'a> {
    pub quoted_markdown: &'a str,
}

/// What a saved description change does to the thread.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RevisionDecision {
    Open { quoted: String, body: String },
    Update { body: String },
    Remove,
    Ignore,
}

/// A task has been evaluated once it has a wrapup, a step past `OPEN`, or a review line past `OPEN`;
/// before that the description is a brief being written.
pub fn is_evaluated<'a>(has_wrapup: bool, step_states: impl IntoIterator<Item = &'a TaskStepState>, review_state: TaskStepState) -> bool {
    let step_started = step_states.into_iter().any(|state| !state.unstarted());
    has_wrapup || step_started || review_state != TaskStepState::Open
}

/// A task with a plan has its revisions applied to the plan until the person says otherwise.
pub fn default_target(task_has_steps: bool) -> FeedbackKind {
    if task_has_steps { FeedbackKind::Plan } else { FeedbackKind::Implementation }
}

/// A blank new description is ignored: a revision's body cannot be blank.
pub fn decide(evaluated: bool, open_revision: Option<OpenRevision>, before: Option<&str>, after: Option<&str>) -> RevisionDecision {
    let (before, after) = (before.unwrap_or_default(), after.unwrap_or_default());
    if !evaluated || before == after || after.trim().is_empty() {
        return RevisionDecision::Ignore;
    }
    match open_revision {
        Some(open) if open.quoted_markdown == after => RevisionDecision::Remove,
        Some(_) => RevisionDecision::Update { body: after.to_string() },
        None => RevisionDecision::Open { quoted: before.to_string(), body: after.to_string() },
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/feedback/rule/revision_rule_tests.rs"]
mod tests;
