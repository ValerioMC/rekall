use rekall_core::settings::FolderInspection;
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

impl From<FolderInspection> for CheckResponse {
    fn from(inspection: FolderInspection) -> Self {
        let usable = inspection.usable();
        Self {
            resolved_path: inspection.resolved_path,
            exists: inspection.exists,
            is_directory: inspection.is_directory,
            writable: inspection.writable,
            has_database: inspection.has_database,
            usable,
        }
    }
}
