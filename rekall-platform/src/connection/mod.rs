//! Whether a connection's stored credential still opens the door it is for, asked from the console
//! before a session finds out mid-deploy (`POST /api/connections/{id}/check`), and the Jira search
//! the console imports issues through.

mod atlassian_document;
mod connection_check;
mod connection_check_controller;
mod connection_check_service;
mod connection_probe;
mod http_connection_probe;
mod http_jira_issues;
mod probe_outcome;

pub use connection_check::ConnectionCheck;
pub use connection_check_controller::routes;
pub use connection_check_service::ConnectionCheckService;
pub use connection_probe::ConnectionProbe;
pub use http_connection_probe::HttpConnectionProbe;
pub use http_jira_issues::HttpJiraIssues;
pub use probe_outcome::ProbeOutcome;

pub(crate) use http_connection_probe::failure_chain;
