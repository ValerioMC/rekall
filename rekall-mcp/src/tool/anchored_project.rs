use rekall_core::ProjectRef;

use crate::protocol::ToolError;

use super::Anchor;

/// A target that is one project: `project:<label>`, with its `company:<name>` beside it when two
/// companies share the label, and the task a session is on allowed beside it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnchoredProject {
    pub project: ProjectRef,
}

impl AnchoredProject {
    pub fn from(anchors: &[Anchor]) -> Result<Self, ToolError> {
        let projects: Vec<&Anchor> = anchors.iter().filter(|anchor| anchor.is("project")).collect();
        let companies: Vec<&Anchor> = anchors.iter().filter(|anchor| anchor.is("company")).collect();
        if let Some(stray) = anchors.iter().find(|anchor| !anchor.is("project") && !anchor.is("task") && !anchor.is("company")) {
            return Err(ToolError::Failure(format!("`{stray}` does not name a project. Pass `project:<label>`.")));
        }
        let project = match projects.as_slice() {
            [project] => project,
            [] => return Err(ToolError::Failure("No project in those anchors. Pass `project:<label>`.".into())),
            _ => return Err(ToolError::Failure("Two project anchors were given. This belongs to one project.".into())),
        };
        match companies.as_slice() {
            [] => Ok(Self { project: ProjectRef::bare(&project.value) }),
            [company] => Ok(Self { project: ProjectRef::in_company(&company.value, &project.value) }),
            _ => Err(ToolError::Failure("Two company anchors were given. A project belongs to one company.".into())),
        }
    }

    /// The project the anchors name and nothing else: `project:<label>`, or with its company.
    pub fn alone(anchors: &[Anchor]) -> Option<ProjectRef> {
        match anchors {
            [project] if project.is("project") => Some(ProjectRef::bare(&project.value)),
            [company, project] | [project, company] if company.is("company") && project.is("project") => {
                Some(ProjectRef::in_company(&company.value, &project.value))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/tool/anchored_project_tests.rs"]
mod tests;
