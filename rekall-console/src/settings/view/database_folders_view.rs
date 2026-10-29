use rekall_core::settings::SetupStatus;

use crate::settings::DatabaseView;

pub struct DatabaseFoldersView {
    pub status: SetupStatus,
    pub active: Option<DatabaseView>,
    pub databases: Vec<DatabaseView>,
}
