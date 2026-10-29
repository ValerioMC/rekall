//! Search across the catalog.

mod controller;
mod request;
mod routes;

pub use request::search_query::SearchQuery;
pub(crate) use routes::routes;
