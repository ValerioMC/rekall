//! Notes and the tasks they are attached to.

mod controller;
mod request;
mod response;
mod routes;

pub use request::document_list_query::DocumentListQuery;
pub use request::document_request::DocumentRequest;
pub use response::document_response::DocumentResponse;
pub use request::document_search_query::DocumentSearchQuery;
pub(crate) use routes::routes;
