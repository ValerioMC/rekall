use rekall_model::TaskKind;

/// What a session asks a new task to be: its names, its description, a task or an epic, and the
/// label of the epic it is filed under.
#[derive(Clone, Debug)]
pub struct TaskBrief {
    pub title: Option<String>,
    pub label: Option<String>,
    pub description: Option<String>,
    pub kind: TaskKind,
    pub epic: Option<String>,
}
