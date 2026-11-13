use rekall_kernel::Id;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskSentBackView {
    pub task_id: Id,
    pub code: String,
    pub title: String,
    pub sent_back: usize,
    pub passes_total: usize,
}
