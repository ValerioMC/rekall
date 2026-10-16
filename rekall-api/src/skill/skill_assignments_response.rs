use rekall_common::Id;
use rekall_model::{tag_skill, task_skill};
use serde::Serialize;

/// One skill on a task (`stepId` null) or on one of its steps.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskSkillResponse {
    pub task_id: Id,
    pub step_id: Option<Id>,
    pub skill_name: String,
}

impl TaskSkillResponse {
    pub fn of(skill: &task_skill::Model) -> Self {
        Self { task_id: skill.task_id, step_id: skill.step_id, skill_name: skill.skill_name.clone() }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TagSkillResponse {
    pub tag_id: Id,
    pub skill_name: String,
}

impl TagSkillResponse {
    pub fn of(skill: &tag_skill::Model) -> Self {
        Self { tag_id: skill.tag_id, skill_name: skill.skill_name.clone() }
    }
}

/// Every assignment, in rank order within each owner: what the console loads once.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillAssignmentsResponse {
    pub task_skills: Vec<TaskSkillResponse>,
    pub tag_skills: Vec<TagSkillResponse>,
}
