//! `RestExceptionHandler`: every failure a handler returns becomes an RFC 7807 problem, with the
//! status the Java handler chose for its exception class and the request path as `instance`, the
//! way Spring filled it in. Failures Spring raised before a controller ran (a missing query
//! parameter, a path nothing answers, a method nothing takes) keep the shape Spring Boot's error
//! controller gave them instead: `timestamp`, `status`, `error`, `path`.

mod api_error;
mod error_renderer;
mod framework_error;
mod problem;

pub use api_error::{ApiResult, ApiError};
pub use error_renderer::render_errors;
pub use framework_error::{FrameworkError, framework};
pub use problem::Problem;
