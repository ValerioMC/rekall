use std::sync::Arc;

use rekall_claude::usage::UsageReader;
use rekall_core::connection::{JiraIssues, SecretStore};
use rekall_core::vulnerability::AdvisoryFeed;
use rekall_core::Clock;

use rekall_platform::connection::ConnectionProbe;
use rekall_platform::version::ReleaseFeed;

/// What a caller, usually a test, can swap in for the real collaborators.
#[derive(Clone, Default)]
pub struct StartOptions {
    /// Stands in for the Claude usage service.
    pub usage: Option<Arc<dyn UsageReader>>,
    /// Stands in for the system clock.
    pub clock: Option<Clock>,
    /// Stands in for GitHub as the source of the latest release.
    pub release_feed: Option<Arc<dyn ReleaseFeed>>,
    /// Stands in for the OS credential store, so a test never touches the real keychain.
    pub secrets: Option<Arc<dyn SecretStore>>,
    /// Stands in for GitLab, the clusters and the Jira sites a connection check asks.
    pub connection_probe: Option<Arc<dyn ConnectionProbe>>,
    /// Stands in for the Jira sites issues are imported from.
    pub jira_issues: Option<Arc<dyn JiraIssues>>,
    /// Stands in for the GitHub Advisory Database a vulnerability scan reads.
    pub advisory_feed: Option<Arc<dyn AdvisoryFeed>>,
    /// Hand the instance a restarter nothing listens to: a database switch is recorded but the
    /// application stays on the database it has.
    pub no_restart: bool,
}
