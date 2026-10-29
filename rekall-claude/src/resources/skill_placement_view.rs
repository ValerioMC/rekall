use rekall_store::skill_placement::Model;
use serde::Serialize;

/// One skill and the Rekall-only group it is shown under.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillPlacementView {
    pub skill_path: String,
    pub group: String,
}

impl From<Model> for SkillPlacementView {
    fn from(row: Model) -> Self {
        Self { skill_path: row.skill_path, group: row.group_label }
    }
}
