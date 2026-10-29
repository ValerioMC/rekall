use std::sync::Arc;

use rekall_core::claude::{TerminalHost, TerminalPipe};
use rekall_core::connection::{JiraIssues, SecretStore};
use rekall_core::queue::QueueRunner;

/// What the routes reach on the machine, each behind a port of `rekall-core`. `rekall-app` builds
/// the implementations and hands them over here.
#[derive(Clone)]
pub struct ApiPorts {
    pub secrets: Arc<dyn SecretStore>,
    pub jira: Arc<dyn JiraIssues>,
    pub terminals: Arc<dyn TerminalHost>,
    pub terminal_pipe: Arc<dyn TerminalPipe>,
    pub queue_runner: Arc<dyn QueueRunner>,
}
