//! The version this build is and whether a newer release exists, served by `rekall_console::version`.

mod routes;
mod controller;
mod request;

pub(crate) use routes::routes;

use request::version_query::VersionQuery;
