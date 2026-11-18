use rekall_kernel::Id;

/// What turning a task into an epic would delete: the parts of a task an epic cannot hold.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EpicConversionView {
    pub task_id: Id,
    pub steps: usize,
    pub steps_done: usize,
    pub routes: usize,
    pub has_wrapup: bool,
    pub commits: usize,
    pub deviations: usize,
    pub waits_for: usize,
    pub needed_by: usize,
    pub skills: usize,
    pub has_pull_request: bool,
    pub epic_code: Option<String>,
    pub epic_title: Option<String>,
    pub queued: bool,
}
