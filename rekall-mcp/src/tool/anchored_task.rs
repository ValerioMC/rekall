use crate::protocol::ToolError;

use super::Anchor;

/// A write's target: exactly one task, with the project that disambiguates it when given.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnchoredTask {
    pub project_label: Option<String>,
    pub task_label: String,
}

impl AnchoredTask {
    pub fn from(anchors: &[Anchor]) -> Result<Self, ToolError> {
        let mut project = None;
        let mut task: Option<String> = None;
        let mut bare = Vec::new();
        for anchor in anchors {
            if anchor.is("task") {
                if task.is_some() {
                    return Err(ToolError::Failure("Two task anchors were given. This write belongs to one task.".into()));
                }
                task = Some(anchor.value.clone());
            } else if anchor.is("project") {
                project = Some(anchor.value.clone());
            } else if !anchor.is_qualified() {
                bare.push(anchor.value.clone());
            } else {
                return Err(ToolError::Failure(format!(
                    "`{anchor}` cannot say which task to write to. Pass `project:<label> task:<label>`."
                )));
            }
        }
        if task.is_none() && bare.len() == 1 {
            task = bare.pop();
        }
        let Some(task) = task else {
            return Err(ToolError::Failure(
                "No task in those anchors. This write belongs to exactly one task: pass `project:<label> task:<label>`."
                    .into(),
            ));
        };
        Ok(Self { project_label: project, task_label: task })
    }

    pub fn anchor(&self) -> String {
        match &self.project_label {
            None => format!("task:{}", self.task_label),
            Some(project) => format!("project:{project} task:{}", self.task_label),
        }
    }
}
