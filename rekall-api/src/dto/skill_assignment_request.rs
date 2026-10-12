use serde::Deserialize;

/// The whole ordered list of skill names an owner should carry; a missing list is an empty one.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SkillAssignmentRequest {
    pub skills: Vec<String>,
}
