use rekall_core::settings::SetupStatus;

use super::DatabaseView;

pub struct DatabaseFoldersView {
    pub status: SetupStatus,
    pub active: Option<DatabaseView>,
    pub databases: Vec<DatabaseView>,
}
