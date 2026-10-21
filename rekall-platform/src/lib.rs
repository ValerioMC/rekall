//! What the application does around the catalog, apart from serving it: which database to open
//! (`bootstrap`), backups and restores (`backup`), restarting onto another database (`restart`),
//! the running version and the check for a newer release (`version`), the OS credential store
//! connection credentials live in (`secret`). `rekall-app` wires it.

pub mod backup;
pub mod bootstrap;
pub mod config;
pub mod restart;
pub mod secret;
pub mod version;
