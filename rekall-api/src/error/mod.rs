//! Every failure a handler returns becomes an RFC 9457 problem, its status chosen by the kind
//! of `RekallError` and the request path as `instance`. Failures raised before a handler runs (a
//! missing query parameter, a path nothing answers, a method nothing takes) are a small JSON
//! status instead: `timestamp`, `status`, `error`, `path`.

mod api_error;
mod error_renderer;
mod framework_error;
mod problem;

pub use api_error::{ApiResult, ApiError};
pub use error_renderer::render_errors;
pub use framework_error::{FrameworkError, framework};
pub use problem::Problem;
