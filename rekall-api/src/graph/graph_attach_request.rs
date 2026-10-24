use rekall_common::Id;
use serde::Deserialize;

/// The graph to lay on a task.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GraphAttachRequest {
    pub graph_id: Option<Id>,
}
