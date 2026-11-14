use rekall_kernel::Id;
use serde::Deserialize;

/// A commit named by hand: the hash the picker chose or the person pasted. The step is optional,
/// and so is the component: none is the project's own folder.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PickedCommitReferenceRequest {
    pub step_id: Option<Id>,
    pub commit_hash: Option<String>,
    pub component_id: Option<Id>,
}
