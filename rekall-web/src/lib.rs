//! The HTTP vocabulary `rekall-api` and `rekall-infra` share: `ApiError` and its problem body,
//! the request extractors (`JsonBody`, `Validator`, path ids) and the `created` / `no_content`
//! response helpers. It knows no service, so neither router depends on the other.

pub mod error;
pub mod extract;
mod response;

pub use error::{ApiError, ApiResult};
pub use response::{created, no_content, png};
