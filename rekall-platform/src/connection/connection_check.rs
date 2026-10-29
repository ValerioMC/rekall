use rekall_kernel::Instant;
use serde::Serialize;

/// The answer the console shows beside a connection: whether its credential works, and in words.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionCheck {
    pub ok: bool,
    pub detail: String,
    pub checked_at: Instant,
}
