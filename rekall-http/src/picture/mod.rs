//! Keeping a picture beside the database and serving it back, served by `rekall_console::picture`.

mod controller;
mod request;
mod response;
mod routes;

pub(crate) use routes::routes;
