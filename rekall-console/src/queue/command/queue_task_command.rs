use rekall_core::queue::QueueLaunch;
use rekall_kernel::Id;

pub struct QueueTaskCommand {
    pub task_id: Id,
    pub launch: QueueLaunch,
}
