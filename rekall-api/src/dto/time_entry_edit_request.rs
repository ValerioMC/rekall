use rekall_common::Instant;
use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TimeEntryEditRequest {
    pub started_at: Option<Instant>,
    pub stopped_at: Option<Instant>,
}
