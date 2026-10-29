//! The database folders a person has registered in `config.json`, which one is open, and the
//! restart that moves the application onto another. The registry file, a folder's inspection and
//! the restart are ports that `rekall-infra` implements.

mod model;
mod port;

pub use port::application_restart::ApplicationRestart;
pub use model::database_entry::DatabaseEntry;
pub use port::database_folders::DatabaseFolders;
pub use model::database_registry::DatabaseRegistry;
pub use model::folder_inspection::FolderInspection;
pub use model::setup_status::SetupStatus;
