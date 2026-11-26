//! Keeping a picture beside the database, serving it back and deleting the unused ones, served by
//! `rekall_console::picture`.

mod controller;
mod request;
mod response;
mod routes;

pub(crate) use routes::routes;
