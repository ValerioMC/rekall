use rekall_kernel::{Id, Instant};
use serde::Serialize;

use super::ChangedFileView;

/// What one commit logged against a task changed. `available` is false when neither git nor the
/// stored diff could produce its patch; `truncated` when the patch was cut at the size cap.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitChangesView {
    pub reference_id: Id,
    pub commit_hash: String,
    pub subject: String,
    pub step_id: Option<Id>,
    pub logged_at: Instant,
    pub available: bool,
    pub truncated: bool,
    pub files: Vec<ChangedFileView>,
}
