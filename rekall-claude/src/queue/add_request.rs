use rekall_kernel::Id;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct AddRequest {
    pub task_id: Option<Id>,
}
