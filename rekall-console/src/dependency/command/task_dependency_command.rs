use rekall_kernel::Id;

/// The task another one is to wait for.
#[derive(Clone, Debug)]
pub struct TaskDependencyCommand {
    pub depends_on_task_id: Id,
}
