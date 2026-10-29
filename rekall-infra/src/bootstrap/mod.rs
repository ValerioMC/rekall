//! Where the database lives: the registry of database
//! folders in `~/.rekall/config.json`, the choice made at start-up, and the `DatabaseFolders`
//! port the settings screen changes it through.

mod database_folder_registry;

pub mod folder;
pub mod location;
pub mod registry;

pub use database_folder_registry::DatabaseFolderRegistry;
