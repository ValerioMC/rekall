use crate::config::DatabaseOverride;

use rekall_core::settings::SetupStatus;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Resolved {
    pub database: DatabaseOverride,
    pub status: SetupStatus,
}
