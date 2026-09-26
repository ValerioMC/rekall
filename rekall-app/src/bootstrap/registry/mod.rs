//! `~/.rekall/config.json`: every database folder a person has registered and which one is in
//! use. The file is shared with the Java build, which reads it with Jackson, so the shape is the
//! records' own: `{"activeId": ..., "databases": [{"id", "label", "path", "addedAt", "lastUsedAt"}]}`.

mod database_entry;
mod database_registry;
mod database_registry_store;

pub use database_entry::DatabaseEntry;
pub use database_registry::DatabaseRegistry;
pub use database_registry_store::DatabaseRegistryStore;
