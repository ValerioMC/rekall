//! The version this build is and whether a newer release exists, served by `rekall_console::version`.

mod routes;
mod version_controller;
mod version_query;

pub(crate) use routes::routes;

use version_query::VersionQuery;
