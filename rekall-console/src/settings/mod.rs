//! The database folders as the setup screen and the Settings panel manage them. Adding or switching
//! one rewrites the registry and restarts onto it, through the ports of `rekall_core::settings`.

mod add_database_command;
mod added_database_view;
mod database_folder_service;
mod database_folders_view;
mod database_view;

pub use add_database_command::AddDatabaseCommand;
pub use added_database_view::AddedDatabaseView;
pub use database_folder_service::DatabaseFolderService;
pub use database_folders_view::DatabaseFoldersView;
pub use database_view::DatabaseView;
