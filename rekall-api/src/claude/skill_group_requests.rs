use serde::Deserialize;

/// Puts one of the person's skills under a group, or out of its group when none is named.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceSkillRequest {
    pub skill_path: Option<String>,
    pub group: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RenameGroupRequest {
    pub from: Option<String>,
    pub to: Option<String>,
}
