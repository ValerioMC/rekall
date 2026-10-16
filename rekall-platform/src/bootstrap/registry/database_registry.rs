use serde::{Deserialize, Serialize};

use super::DatabaseEntry;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseRegistry {
    pub active_id: Option<String>,
    #[serde(default)]
    pub databases: Vec<DatabaseEntry>,
}

impl DatabaseRegistry {
    pub fn empty() -> Self {
        Self { active_id: None, databases: Vec::new() }
    }

    pub fn active(&self) -> Option<&DatabaseEntry> {
        let active = self.active_id.as_deref()?;
        self.databases.iter().find(|entry| entry.id == active)
    }
}
