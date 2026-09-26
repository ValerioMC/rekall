use rekall_model::{company, project, tag, task, task_step, wrapup};

/// Everything a task row is built from.
pub struct TaskGraph<'a> {
    pub task: &'a task::Model,
    pub project: &'a project::Model,
    pub company: &'a company::Model,
    pub steps: Vec<&'a task_step::Model>,
    pub document_count: usize,
    pub wrapup: Option<&'a wrapup::Model>,
    pub tag: Option<&'a tag::Model>,
}
