use serde::Serialize;

/// How many exports a clear deleted.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct ProjectExportClearedResponse {
    pub deleted: u64,
}
