//! What every controller shares: `ApiError` and its problem body, the request extractors
//! (`JsonBody`, `Validator`, path ids) and the `created` / `no_content` response helpers.

pub mod error;
pub mod extract;
mod response;

pub use error::{ApiError, ApiResult};
pub use response::{created, no_content, png};
