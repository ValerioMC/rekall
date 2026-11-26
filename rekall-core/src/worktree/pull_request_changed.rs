use rekall_kernel::Id;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestChanged {
    pub task_id: Id,
}

impl crate::StreamEvent for PullRequestChanged {
    const FRAME: &'static str = "pull-request";
}
