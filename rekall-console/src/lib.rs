//! The services only the console calls, one folder per domain. A service takes a command and
//! returns a view, both defined beside it; the HTTP layer maps its requests and responses onto them.

pub mod catalog;
pub mod component;
pub mod connection;
pub mod diagram;
pub mod filesystem;
pub mod graph;
pub mod note;
pub mod section;
pub mod skill;
pub mod tag;
pub mod vulnerability;
