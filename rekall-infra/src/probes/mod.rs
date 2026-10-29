//! Whether a connection's stored credential still opens the door it is for, asked over HTTP: the
//! `ConnectionProbe` port of `rekall_core::connection`.

mod http_connection_probe;

pub use http_connection_probe::HttpConnectionProbe;

pub(crate) use http_connection_probe::failure_chain;
