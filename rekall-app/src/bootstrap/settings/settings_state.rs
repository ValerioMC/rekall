use super::super::registry::DatabaseRegistryStore;
use crate::restart::Restarter;

#[derive(Clone)]
pub struct SettingsState {
    pub store: DatabaseRegistryStore,
    pub user_home: std::path::PathBuf,
    pub restarter: Restarter,
}
