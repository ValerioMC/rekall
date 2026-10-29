use rekall_common::Id;
use rekall_model::tag_skill;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagSkillView {
    pub tag_id: Id,
    pub skill_name: String,
}

impl TagSkillView {
    pub fn of(skill: &tag_skill::Model) -> Self {
        Self { tag_id: skill.tag_id, skill_name: skill.skill_name.clone() }
    }
}
