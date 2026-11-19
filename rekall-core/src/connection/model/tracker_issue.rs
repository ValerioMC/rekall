use rekall_store::{task, task_tracker_link};

/// What Rekall pushes of a task to the project's task manager: its title and its markdown description.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrackerIssue {
    pub title: String,
    pub description: String,
}

impl TrackerIssue {
    /// The task as the person wrote it, title and description.
    pub fn of(task: &task::Model) -> Self {
        Self { title: task_tracker_link::issue_title(&task.title), description: task.description.clone().unwrap_or_default() }
    }

    /// The task's title alone: what the task manager is told when the task is created.
    pub fn titled(task: &task::Model) -> Self {
        Self { description: String::new(), ..Self::of(task) }
    }

    pub fn fingerprint(&self) -> String {
        task_tracker_link::content_fingerprint(&self.title, &self.description)
    }
}
