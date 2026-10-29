//! Search across the catalog.

mod search_controller;
mod search_query;
mod routes;

pub use search_query::SearchQuery;
pub(crate) use routes::routes;
