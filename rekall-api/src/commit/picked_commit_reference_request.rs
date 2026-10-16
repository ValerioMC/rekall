use rekall_common::Id;
use serde::Deserialize;

/// A commit named by hand: the hash the picker chose or the person pasted; the step is optional.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PickedCommitReferenceRequest {
    pub step_id: Option<Id>,
    pub commit_hash: Option<String>,
}
