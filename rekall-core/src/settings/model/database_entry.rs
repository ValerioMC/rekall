use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseEntry {
    pub id: String,
    pub label: String,
    pub path: String,
    pub added_at: String,
    pub last_used_at: String,
}
