use rekall_common::Id;
use rekall_diagram::SemanticGraph;

/// A diagram a session generated for a task, addressed by the task's anchors. It is stored on
/// that task's project with the task as its origin. `diagram_id` replaces that diagram instead.
#[derive(Clone, Debug)]
pub struct GeneratedDiagram {
    pub project_label: Option<String>,
    pub task_label: String,
    pub diagram_id: Option<Id>,
    pub title: String,
    pub question: String,
    pub graph: SemanticGraph,
}
