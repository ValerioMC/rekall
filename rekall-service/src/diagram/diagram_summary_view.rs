use rekall_common::{Id, Instant, InternalFailure, RekallError, Result};
use rekall_diagram::DiagramKind;
use rekall_model::diagram;
use serde::Serialize;

/// A diagram as a list shows it: everything but its sheets, each sheet's type, and whether one of
/// them plays as a walkthrough.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagramSummaryView {
    pub id: Id,
    pub project_id: Id,
    pub task_id: Option<Id>,
    pub title: String,
    pub question: String,
    pub kinds: Vec<DiagramKind>,
    pub walkthrough: bool,
    pub created_at: Instant,
    pub updated_at: Instant,
}

impl DiagramSummaryView {
    pub fn of(row: &diagram::Model) -> Result<Self> {
        Ok(Self {
            id: row.id,
            project_id: row.project_id,
            task_id: row.task_id,
            title: row.title.clone(),
            question: row.question.clone(),
            kinds: stored_kinds(row)?,
            walkthrough: row.walkthrough,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    /// The column form of a document's kinds: `sequence,state`.
    pub fn column_of(kinds: &[DiagramKind]) -> String {
        kinds.iter().map(|kind| kind.name()).collect::<Vec<_>>().join(",")
    }
}

/// Every name was written from a [`DiagramKind`]; one that no longer reads is corruption.
fn stored_kinds(row: &diagram::Model) -> Result<Vec<DiagramKind>> {
    row.kinds
        .split(',')
        .map(|name| {
            DiagramKind::of_name(name).ok_or_else(|| {
                RekallError::internal(InternalFailure::CorruptData, format!("Diagram {} has the unknown kind {name:?}", row.id))
            })
        })
        .collect()
}
