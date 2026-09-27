use rekall_common::Id;
use rekall_diagram::TraceHit;
use serde::Serialize;

/// The elements of one diagram a line of code belongs to.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagramTraceView {
    pub diagram_id: Id,
    pub title: String,
    pub hits: Vec<TraceHit>,
}
