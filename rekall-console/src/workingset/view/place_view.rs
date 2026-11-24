use rekall_kernel::Id;

/// A place inside a project: one of its tasks, or its task list when `task_id` is empty.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlaceView {
    pub project_id: Id,
    pub task_id: Option<Id>,
}
