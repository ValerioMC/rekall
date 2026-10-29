//! The services only the console calls, one folder per domain. A service takes a command and
//! returns a view, both defined beside it; the HTTP layer maps its requests and responses onto them.

pub mod connection;
