use rekall_console::adherence::TaskSentBackView;
use rekall_kernel::Id;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskSentBackResponse {
    pub task_id: Id,
    pub code: String,
    pub title: String,
    pub sent_back: usize,
    pub passes_total: usize,
}

impl From<TaskSentBackView> for TaskSentBackResponse {
    fn from(view: TaskSentBackView) -> Self {
        Self { task_id: view.task_id, code: view.code, title: view.title, sent_back: view.sent_back, passes_total: view.passes_total }
    }
}
