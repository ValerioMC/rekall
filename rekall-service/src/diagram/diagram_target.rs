/// Where a session's diagram goes, as its anchors name it: a project alone, or a task, which
/// puts it on the task's project with the task as its origin.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiagramTarget {
    Project { label: String },
    Task { project_label: Option<String>, task_label: String },
}
