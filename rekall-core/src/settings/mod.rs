//! The database folders a person has registered in `config.json`, which one is open, and the
//! restart that moves the application onto another. The registry file, a folder's inspection and
//! the restart are ports that `rekall-infra` implements.

mod application_restart;
mod database_entry;
mod database_folders;
mod database_registry;
mod folder_inspection;
mod setup_status;

pub use application_restart::ApplicationRestart;
pub use database_entry::DatabaseEntry;
pub use database_folders::DatabaseFolders;
pub use database_registry::DatabaseRegistry;
pub use folder_inspection::FolderInspection;
pub use setup_status::SetupStatus;
