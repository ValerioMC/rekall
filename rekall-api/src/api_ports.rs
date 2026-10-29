use std::sync::Arc;

use rekall_core::claude::{ClaudeCodeSetup, ClaudeResources, TerminalHost, TerminalPipe, UsageReader};
use rekall_core::connection::{CredentialDesk, JiraIssues, SecretStore};
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
    pub usage: Arc<dyn UsageReader>,
    pub claude_folder: Arc<dyn ClaudeResources>,
    pub claude_setup: Arc<dyn ClaudeCodeSetup>,
    pub credential_desk: Arc<dyn CredentialDesk>,
}
