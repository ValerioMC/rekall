/// One skill to assign: its name and the free text it is invoked with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkillCommand {
    pub name: String,
    pub arguments: Option<String>,
}
