//! Where the database lives and how Claude Code is told about Rekall: the registry of database
//! folders in `~/.rekall/config.json`, the choice made at start-up, the settings endpoints that
//! change it, and the Claude Code registration.

pub mod folder;
pub mod installer;
pub mod location;
pub mod registry;
pub mod settings;
