//! The Settings screen's route for where Claude Code's traces are sent.

mod controller;
mod request;
mod routes;

pub use request::telemetry_settings_request::TelemetrySettingsRequest;
pub use routes::routes;
