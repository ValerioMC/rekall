use rekall_console::connection::DiscoveryOutcome;
use serde::Serialize;

/// What discovering a repository did with it, as the console reads it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DiscoveryOutcomeResponse {
    Bound,
    AlreadyBound,
    NoOrigin,
    NoConnection,
    NoCredential,
}

impl From<DiscoveryOutcome> for DiscoveryOutcomeResponse {
    fn from(outcome: DiscoveryOutcome) -> Self {
        match outcome {
            DiscoveryOutcome::Bound => Self::Bound,
            DiscoveryOutcome::AlreadyBound => Self::AlreadyBound,
            DiscoveryOutcome::NoOrigin => Self::NoOrigin,
            DiscoveryOutcome::NoConnection => Self::NoConnection,
            DiscoveryOutcome::NoCredential => Self::NoCredential,
        }
    }
}
