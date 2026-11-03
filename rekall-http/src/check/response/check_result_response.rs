use rekall_console::check::CheckResultView;
use rekall_kernel::{Id, Instant};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckResultResponse {
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

impl From<CheckResultView> for CheckResultResponse {
    fn from(view: CheckResultView) -> Self {
        Self {
            id: view.id,
            check_id: view.check_id,
            task_id: view.task_id,
            step_id: view.step_id,
            label: view.label,
            command: view.command,
            passed: view.passed,
            timed_out: view.timed_out,
            exit_code: view.exit_code,
            output_tail: view.output_tail,
            ran_at: view.ran_at,
            duration_ms: view.duration_ms,
        }
    }
}
