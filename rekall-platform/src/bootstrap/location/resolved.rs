use crate::config::DatabaseOverride;

use super::SetupStatus;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Resolved {
    pub database: DatabaseOverride,
    pub status: SetupStatus,
}
