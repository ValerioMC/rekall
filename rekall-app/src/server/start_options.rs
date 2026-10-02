use std::sync::Arc;

use rekall_claude::usage::UsageReader;
use rekall_service::Clock;

use crate::version::ReleaseFeed;

/// What a caller can swap in, as the tests' `@MockitoBean`s did.
#[derive(Clone, Default)]
pub struct StartOptions {
    /// Stands in for the Claude usage service.
    pub usage: Option<Arc<dyn UsageReader>>,
    /// Stands in for the system clock.
    pub clock: Option<Clock>,
    /// Stands in for GitHub as the source of the latest release.
    pub release_feed: Option<Arc<dyn ReleaseFeed>>,
    /// Hand the instance a restarter nothing listens to, as a Spring test context had: a
    /// database switch is recorded but the application stays on the database it has.
    pub no_restart: bool,
}
