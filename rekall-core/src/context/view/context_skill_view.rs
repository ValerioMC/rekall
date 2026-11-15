/// Where a skill was assigned, which decides when a session invokes it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContextSkillScope {
    Task,
    /// By the task's tag, named here.
    Tag(String),
    /// By one step, named here; only while that step is worked.
    Step(String),
}

/// One invocation of a skill with the Skill tool, and why it was handed over. A skill is told apart
/// by its name and its arguments together.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContextSkillView {
    pub name: String,
    pub arguments: Option<String>,
    pub scope: ContextSkillScope,
}
