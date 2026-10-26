use rekall_model::task;

/// A task of the project as a `[[task:label]]` in another task's text names it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskReferenceView {
    pub label: String,
    pub title: String,
}

impl TaskReferenceView {
    pub fn of(row: &task::Model) -> Self {
        Self { label: row.label.clone(), title: row.title.clone() }
    }
}
