use rekall_kernel::Instant;
use serde::Deserialize;

/// `startAt` null starts at once.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::queue) struct StartRequest {
    pub start_at: Option<Instant>,
}
