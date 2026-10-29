use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CheckResponse {
    pub(super) resolved_path: String,
    pub(super) exists: bool,
    pub(super) is_directory: bool,
    pub(super) writable: bool,
    pub(super) has_database: bool,
    pub(super) usable: bool,
}
