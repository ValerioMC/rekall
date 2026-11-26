use rekall_kernel::{Id, Instant};
use serde::Serialize;

use super::ChangedFileView;

/// What one commit logged against a task changed. `available` is false when neither git nor the
/// stored diff could produce its patch; `truncated` when the patch was cut at the size cap.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitChangesView<Files = Vec<ChangedFileView>> {
    pub reference_id: Id,
    pub commit_hash: String,
    pub subject: String,
    pub step_id: Option<Id>,
    pub logged_at: Instant,
    pub available: bool,
    pub truncated: bool,
    pub files: Files,
}

impl<Files> CommitChangesView<Files> {
    pub(in crate::commit) fn map_files<Mapped>(self, map: impl FnOnce(Files) -> Mapped) -> CommitChangesView<Mapped> {
        CommitChangesView {
            reference_id: self.reference_id,
            commit_hash: self.commit_hash,
            subject: self.subject,
            step_id: self.step_id,
            logged_at: self.logged_at,
            available: self.available,
            truncated: self.truncated,
            files: map(self.files),
        }
    }
}
