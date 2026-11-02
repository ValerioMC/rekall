use rekall_kernel::Id;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::queue) struct RunRequest {
    pub task_id: Option<Id>,
}
