//! Every failure becomes an RFC 9457 problem with the request path as `instance`: a handler's
//! with its status chosen by the kind of `RekallError`, and one raised before a handler runs (a
//! missing query parameter, a path nothing answers, a method nothing takes) with its status's
//! reason as the detail.

mod api_error;
mod error_renderer;
mod framework_error;
mod problem;

pub use api_error::{ApiResult, ApiError};
pub use error_renderer::render_errors;
pub use framework_error::{FrameworkError, framework};
pub use problem::Problem;
