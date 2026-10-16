use rekall_common::Id;
use serde::Deserialize;

/// `stepId` is optional: omitted logs the commit against the task itself.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CommitReferenceRequest {
    pub step_id: Option<Id>,
}
