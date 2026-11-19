use rekall_kernel::Id;
use rekall_store::tag_skill;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagSkillView {
    pub tag_id: Id,
    pub skill_name: String,
    pub arguments: Option<String>,
}

impl TagSkillView {
    pub fn of(skill: &tag_skill::Model) -> Self {
        Self { tag_id: skill.tag_id, skill_name: skill.skill_name.clone(), arguments: skill.arguments.clone() }
    }
}
