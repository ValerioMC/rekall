use rekall_kernel::{Id, Instant};
use rekall_store::{done_check, done_check_result};

/// One kept run of a check, with the check's label and command as they were read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckResultView {
    pub id: Id,
    pub check_id: Id,
    pub task_id: Id,
    pub step_id: Option<Id>,
    pub label: String,
    pub command: String,
    pub passed: bool,
    pub timed_out: bool,
    pub exit_code: Option<i32>,
    pub output_tail: String,
    pub ran_at: Instant,
    pub duration_ms: i64,
}

impl CheckResultView {
    pub fn of(result: &done_check_result::Model, check: &done_check::Model) -> Self {
        Self {
            id: result.id,
            check_id: result.check_id,
            task_id: result.task_id,
            step_id: result.step_id,
            label: check.label.clone(),
            command: check.command.clone(),
            passed: result.passed,
            timed_out: !result.passed && result.exit_code.is_none(),
            exit_code: result.exit_code,
            output_tail: result.output_tail.clone(),
            ran_at: result.ran_at,
            duration_ms: result.duration_ms,
        }
    }
}
