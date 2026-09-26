use rekall_common::Instant;
use serde::Deserialize;

/// `startAt` null starts at once.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct StartRequest {
    pub start_at: Option<Instant>,
}
