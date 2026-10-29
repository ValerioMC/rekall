//! The database folders as the setup screen and the Settings panel manage them. Adding or switching
//! one rewrites the registry and restarts onto it, through the ports of `rekall_core::settings`.

mod command;
mod service;
mod view;

pub use command::add_database_command::AddDatabaseCommand;
pub use view::added_database_view::AddedDatabaseView;
pub use service::database_folder_service::DatabaseFolderService;
pub use view::database_folders_view::DatabaseFoldersView;
pub use view::database_view::DatabaseView;
