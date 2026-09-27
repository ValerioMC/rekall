use rekall_common::Id;
use rekall_diagram::SemanticGraph;

/// A diagram to write. No `id` creates one; an `id` replaces that diagram's title, question and
/// graph, and it has to stay on the same project.
#[derive(Clone, Debug)]
pub struct DiagramDraft {
    pub id: Option<Id>,
    pub project_id: Id,
    pub task_id: Option<Id>,
    pub title: String,
    pub question: String,
    pub graph: SemanticGraph,
}
