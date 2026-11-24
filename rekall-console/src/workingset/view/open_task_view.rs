use rekall_kernel::Id;

/// An open task and the project it belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpenTaskView {
    pub task_id: Id,
    pub project_id: Id,
}
