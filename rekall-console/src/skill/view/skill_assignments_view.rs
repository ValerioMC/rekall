use crate::skill::{TagSkillView, TaskSkillView};

/// Every assignment, in rank order within each owner: what the console loads once.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkillAssignmentsView {
    pub task_skills: Vec<TaskSkillView>,
    pub tag_skills: Vec<TagSkillView>,
}
