use rekall_kernel::{jstr, RekallError, Result};
use rekall_store::{task_decision, task_step, Slug};

use crate::deviation::{planned_count, step_at};

pub const MAX_OPTIONS_PER_CALL: usize = 6;
pub const MAX_LINES_PER_CALL: usize = 12;

/// The key a session addresses a decision by: a lowercase slug such as `db-password`.
pub fn key(raw: &str) -> Result<String> {
    let key = jstr::trim(raw);
    if Slug::matches(key) && jstr::len(key) <= task_decision::KEY_MAX_CHARACTERS {
        return Ok(key.to_string());
    }
    Err(RekallError::illegal(format!(
        "A decision key is a lowercase slug such as 'db-password' (letters, digits, '-', '_' or '.', at most {} characters) and '{raw}' is not one.",
        task_decision::KEY_MAX_CHARACTERS
    )))
}

/// The step a decision belongs to, or none for the task as a whole. A task without a checklist has no step to name.
pub fn step_scope(steps: &[task_step::Model], step_number: Option<i32>) -> Result<Option<i32>> {
    let Some(number) = step_number else {
        return Ok(None);
    };
    let planned = planned_count(steps);
    if planned == 0 {
        return Err(RekallError::illegal("This task has no checklist, so a decision belongs to the task as a whole: leave the step out. Nothing was written."));
    }
    step_at(steps, number).map(|_| Some(number)).ok_or_else(|| {
        RekallError::illegal(format!(
            "A decision refers to a step of the plan, and this task's checklist has {planned} step{}: there is no step {number}. Nothing was written.",
            if planned == 1 { "" } else { "s" }
        ))
    })
}

pub fn within_call(options: usize, lines: usize) -> Result<()> {
    if options > MAX_OPTIONS_PER_CALL {
        return Err(RekallError::illegal(format!("A decision offers at most {MAX_OPTIONS_PER_CALL} options in one call and this one sent {options}. Keep the ones worth choosing between.")));
    }
    if lines > MAX_LINES_PER_CALL {
        return Err(RekallError::illegal(format!("A decision quotes at most {MAX_LINES_PER_CALL} conversation lines in one call and this one sent {lines}. Quote the ones that settle it.")));
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/unit/decision/rule/decision_scope_rule_tests.rs"]
mod tests;
