use rekall_common::Id;
use serde::Serialize;

use super::SearchHitKind;

/// One place a search term was found.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    pub kind: SearchHitKind,
    /// The task to open: the one the text is on, or for a note the first task it is on.
    pub task_id: Option<Id>,
    pub step_id: Option<Id>,
    pub document_id: Option<Id>,
    pub title: String,
    /// The anchor of the task the text is on, for a note the one it opens on.
    pub r#where: String,
    pub excerpt: String,
}
