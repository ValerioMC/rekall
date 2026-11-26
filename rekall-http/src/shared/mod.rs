//! What every controller shares: `ApiError` and its problem body, the request extractors
//! (`JsonBody`, `Validator`, path ids) and the `created` / `accepted` / `no_content` response helpers.

pub mod error;
pub mod extract;
mod origin;
mod response;

pub use error::{ApiError, ApiResult};
pub use origin::is_allowed_origin;
pub use response::{accepted, created, no_content, png};
