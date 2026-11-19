use rekall_store::task;

/// A task of the project as a `[[task:code]]` in another task's text names it. The label is what an older reference still carries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskReferenceView {
    pub code: String,
    pub label: String,
    pub title: String,
}

impl TaskReferenceView {
    pub fn of(row: &task::Model) -> Self {
        Self { code: row.code.clone(), label: row.label.clone(), title: row.title.clone() }
    }
}
