//! `~/.rekall/config.json`: every database folder a person has registered and which one is in
//! use. Its shape is fixed, since files written by every earlier release are read as they are:
//! `{"activeId": ..., "databases": [{"id", "label", "path", "addedAt", "lastUsedAt"}]}`.

mod database_registry_store;

pub use database_registry_store::DatabaseRegistryStore;
