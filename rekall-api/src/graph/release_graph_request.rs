use rekall_kernel::Id;
use serde::Deserialize;

/// The graph a project releases every task it runs with; none clears the choice.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ReleaseGraphRequest {
    pub graph_id: Option<Id>,
}
