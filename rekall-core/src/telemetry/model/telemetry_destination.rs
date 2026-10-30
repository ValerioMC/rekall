use crate::connection::Secret;

/// An enabled, complete setting: everything a launch needs to point `claude` at the instance.
#[derive(Debug)]
pub struct TelemetryDestination {
    pub host: String,
    pub public_key: String,
    pub secret_key: Secret,
    pub log_content: bool,
}
