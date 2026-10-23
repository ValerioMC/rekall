use crate::protocol::ToolError;

use super::Anchor;

/// A target that is one project: `project:<label>`, with the task a session is on allowed beside it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnchoredProject {
    pub project_label: String,
}

impl AnchoredProject {
    pub fn from(anchors: &[Anchor]) -> Result<Self, ToolError> {
        let projects: Vec<&Anchor> = anchors.iter().filter(|anchor| anchor.is("project")).collect();
        if let Some(stray) = anchors.iter().find(|anchor| !anchor.is("project") && !anchor.is("task")) {
            return Err(ToolError::Failure(format!("`{stray}` does not name a project. Pass `project:<label>`.")));
        }
        match projects.as_slice() {
            [project] => Ok(Self { project_label: project.value.clone() }),
            [] => Err(ToolError::Failure("No project in those anchors. Pass `project:<label>`.".into())),
            _ => Err(ToolError::Failure("Two project anchors were given. This belongs to one project.".into())),
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/tool/anchored_project_tests.rs"]
mod tests;
