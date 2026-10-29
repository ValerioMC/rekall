/// What a session was working on, as its transcript names it: a task by its anchors and, while a
/// step was running, that step by its number or title.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct WorkUnit {
    pub project_label: Option<String>,
    pub task_label: String,
    pub step: Option<String>,
}

impl WorkUnit {
    pub fn task(project_label: Option<&str>, task_label: &str) -> Self {
        Self { project_label: project_label.map(str::to_string), task_label: task_label.to_string(), step: None }
    }

    pub fn on_step(&self, step: Option<&str>) -> Self {
        Self { step: step.map(str::to_string), ..self.clone() }
    }

    pub fn same_task(&self, other: &WorkUnit) -> bool {
        self.project_label == other.project_label && self.task_label == other.task_label
    }
}
