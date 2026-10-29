//! Whether a connection's stored credential still opens the door it is for, asked over HTTP from
//! the console (`POST /api/connections/{id}/check`) before a session finds out mid-deploy.

mod connection_check_controller;
mod http_connection_probe;

pub use connection_check_controller::routes;
pub use http_connection_probe::HttpConnectionProbe;

pub(crate) use http_connection_probe::failure_chain;
