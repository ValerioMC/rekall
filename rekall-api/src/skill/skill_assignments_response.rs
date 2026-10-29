use rekall_common::Id;
use rekall_console::skill::{TaskSkillView, TagSkillView, SkillAssignmentsView};
use serde::Serialize;

/// One skill on a task (`stepId` null) or on one of its steps.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskSkillResponse {
    pub task_id: Id,
    pub step_id: Option<Id>,
    pub skill_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TagSkillResponse {
    pub tag_id: Id,
    pub skill_name: String,
}

/// Every assignment, in rank order within each owner: what the console loads once.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillAssignmentsResponse {
    pub task_skills: Vec<TaskSkillResponse>,
    pub tag_skills: Vec<TagSkillResponse>,
}

impl From<TaskSkillView> for TaskSkillResponse {
    fn from(view: TaskSkillView) -> Self {
        Self {
            task_id: view.task_id,
            step_id: view.step_id,
            skill_name: view.skill_name,
        }
    }
}

impl From<TagSkillView> for TagSkillResponse {
    fn from(view: TagSkillView) -> Self {
        Self {
            tag_id: view.tag_id,
            skill_name: view.skill_name,
        }
    }
}

impl From<SkillAssignmentsView> for SkillAssignmentsResponse {
    fn from(view: SkillAssignmentsView) -> Self {
        Self {
            task_skills: view.task_skills.into_iter().map(TaskSkillResponse::from).collect(),
            tag_skills: view.tag_skills.into_iter().map(TagSkillResponse::from).collect(),
        }
    }
}
