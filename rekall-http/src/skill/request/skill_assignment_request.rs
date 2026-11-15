use rekall_console::skill::SkillCommand;
use serde::Deserialize;

/// The whole ordered list of skills an owner should carry; a missing list is an empty one.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SkillAssignmentRequest {
    pub skills: Vec<SkillRequest>,
}

/// A skill's name and the free text it is invoked with.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillRequest {
    pub name: String,
    pub arguments: Option<String>,
}

impl From<SkillRequest> for SkillCommand {
    fn from(request: SkillRequest) -> Self {
        Self { name: request.name, arguments: request.arguments }
    }
}

impl SkillAssignmentRequest {
    pub fn into_commands(self) -> Vec<SkillCommand> {
        self.skills.into_iter().map(SkillCommand::from).collect()
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/skill/request/skill_assignment_request_tests.rs"]
mod tests;
