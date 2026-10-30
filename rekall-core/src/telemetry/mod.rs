//! Where Claude Code's OpenTelemetry traces are sent, and what a launch passes `claude` so they
//! reach it. Rekall only sets variables for the terminals it opens: it never contacts the
//! destination itself, so an unreachable one cannot affect a session.

mod model;
pub mod rule;
mod service;
mod view;

pub use model::telemetry_change::TelemetryChange;
pub use model::telemetry_destination::TelemetryDestination;
pub use service::telemetry_settings_service::{TelemetrySettingsService, DEFAULT_HOST, SECRET_KEY_ACCOUNT};
pub use view::telemetry_settings_view::TelemetrySettingsView;
