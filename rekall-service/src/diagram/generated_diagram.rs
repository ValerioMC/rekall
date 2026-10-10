use rekall_common::Id;
use rekall_diagram::DiagramDocument;

use super::DiagramTarget;

/// A diagram a session generated, addressed by its anchors. `diagram_id` replaces that diagram
/// instead of creating one.
#[derive(Clone, Debug)]
pub struct GeneratedDiagram {
    pub target: DiagramTarget,
    pub diagram_id: Option<Id>,
    pub title: String,
    pub question: String,
    pub document: DiagramDocument,
}
