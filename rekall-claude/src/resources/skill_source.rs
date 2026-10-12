use serde::Serialize;

/// Who provides a skill: the person, claude.ai's sync, or an installed plugin.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SkillSourceKind {
    Own,
    Synced,
    Plugin,
}

/// Where a skill comes from, for the label next to its lock.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillSource {
    pub kind: SkillSourceKind,
    /// The plugin's name, `claude.ai` for a synced skill, `You` for the person's own.
    pub name: String,
    /// Where the provider got it: a marketplace and version, or the sync's own source tag.
    pub detail: Option<String>,
}

impl SkillSource {
    pub fn own() -> Self {
        Self { kind: SkillSourceKind::Own, name: "You".into(), detail: None }
    }

    pub fn synced(detail: Option<String>) -> Self {
        Self { kind: SkillSourceKind::Synced, name: "claude.ai".into(), detail }
    }

    pub fn plugin(name: &str, detail: Option<String>) -> Self {
        Self { kind: SkillSourceKind::Plugin, name: name.into(), detail }
    }
}
