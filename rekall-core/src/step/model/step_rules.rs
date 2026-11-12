//! What a step's text and state have to satisfy before a service writes them.

use rekall_kernel::{RekallError, Result};
use rekall_store::task_step::{self, MAX_CHARACTERS, TITLE_MAX_CHARACTERS};
use rekall_store::TaskStepState;

use crate::text_rule;

pub(in crate::step) fn awaits_acceptance(step: &task_step::Model) -> bool {
    step.state != TaskStepState::Draft && step.state != TaskStepState::Done
}

pub(in crate::step) fn require_unstarted(step: &task_step::Model) -> Result<()> {
    if step.state.unstarted() {
        return Ok(());
    }
    Err(RekallError::illegal(format!(
        "'{}' is {}. A session deletes a draft or an open step only; a step that was started, claimed or accepted \
         is discarded in the console. Nothing was deleted.",
        step.title,
        step.state.name().to_lowercase()
    )))
}

/// Move a step between `DRAFT` and `OPEN`. Only those two states move; a step a session has
/// started, claimed or finished is refused.
pub(in crate::step) fn apply_draft(step: &mut task_step::Model, draft: bool) -> Result<()> {
    let current = step.state;
    if draft && current == TaskStepState::Open {
        step.mark_state(TaskStepState::Draft);
    } else if !draft && current == TaskStepState::Draft {
        step.mark_state(TaskStepState::Open);
    } else if draft != (current == TaskStepState::Draft) {
        return Err(RekallError::illegal(format!(
            "'{}' is {} and cannot change its draft state. Only a step that is still draft or open moves between the two.",
            step.title,
            current.name().to_lowercase()
        )));
    }
    Ok(())
}

pub(in crate::step) fn validated_title(title: Option<&str>) -> Result<String> {
    let title =
        text_rule::present(title).ok_or_else(|| RekallError::illegal("A step needs a title. To remove one, delete it."))?;
    text_rule::within(title, TITLE_MAX_CHARACTERS, |length| {
        format!(
            "A step's title is capped at {TITLE_MAX_CHARACTERS} characters and this one is {length}. What it has to \
             satisfy goes in the detail below it."
        )
    })
}

/// A step's detail is optional: blank clears it.
pub(in crate::step) fn validated_body(body: Option<&str>) -> Result<Option<String>> {
    text_rule::present(body)
        .map(|body| {
            text_rule::within(body, MAX_CHARACTERS, |length| {
                format!(
                    "A step's detail is capped at {MAX_CHARACTERS} characters and this one is {length}. A step that \
                     long is a task of its own; split it, or move the detail into a note."
                )
            })
        })
        .transpose()
}
