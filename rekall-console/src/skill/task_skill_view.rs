use rekall_kernel::Id;
use rekall_store::task_skill;

/// One skill on a task (`stepId` null) or on one of its steps.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskSkillView {
    pub task_id: Id,
    pub step_id: Option<Id>,
    pub skill_name: String,
}

impl TaskSkillView {
    pub fn of(skill: &task_skill::Model) -> Self {
        Self { task_id: skill.task_id, step_id: skill.step_id, skill_name: skill.skill_name.clone() }
    }
}
