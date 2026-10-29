use rekall_core::settings::FolderInspection;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::settings) struct CheckResponse {
    pub(in crate::settings) resolved_path: String,
    pub(in crate::settings) exists: bool,
    pub(in crate::settings) is_directory: bool,
    pub(in crate::settings) writable: bool,
    pub(in crate::settings) has_database: bool,
    pub(in crate::settings) usable: bool,
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
