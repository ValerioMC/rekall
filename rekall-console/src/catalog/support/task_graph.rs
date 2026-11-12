use rekall_kernel::Id;
use rekall_store::{company, project, tag, task, task_step, task_tracker_link};

/// Everything a task row is built from.
pub struct TaskGraph<'a> {
    pub task: &'a task::Model,
    pub project: &'a project::Model,
    pub company: &'a company::Model,
    pub steps: Vec<&'a task_step::Model>,
    pub document_count: usize,
    pub has_wrapup: bool,
    pub tag: Option<&'a tag::Model>,
    pub depends_on: Vec<Id>,
    pub tracker_link: Option<&'a task_tracker_link::Model>,
}
