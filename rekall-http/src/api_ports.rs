use std::sync::Arc;

use rekall_core::backup::{DatabaseBackups, DatabaseRestore};
use rekall_core::chat::ChatPipe;
use rekall_core::claude::{ClaudeCodeSetup, GuardInstaller, ClaudeResources, TerminalHost, TerminalPipe, UsageReader};
use rekall_core::connection::{ConnectionProbe, CredentialDesk, JiraIssues, SecretStore};
use rekall_core::queue::QueueRunner;
use rekall_core::settings::{ApplicationRestart, DatabaseFolders};
use rekall_core::version::ReleaseFeed;

/// What the routes reach on the machine, each behind a port of `rekall-core`. `rekall-app` builds
/// the implementations and hands them over here.
#[derive(Clone)]
pub struct ApiPorts {
    pub secrets: Arc<dyn SecretStore>,
    pub jira: Arc<dyn JiraIssues>,
    pub terminals: Arc<dyn TerminalHost>,
    pub terminal_pipe: Arc<dyn TerminalPipe>,
    pub chat_pipe: Arc<dyn ChatPipe>,
    pub queue_runner: Arc<dyn QueueRunner>,
    pub usage: Arc<dyn UsageReader>,
    pub claude_folder: Arc<dyn ClaudeResources>,
    pub claude_setup: Arc<dyn ClaudeCodeSetup>,
    pub guard_installer: Arc<dyn GuardInstaller>,
    pub credential_desk: Arc<dyn CredentialDesk>,
    pub connection_probe: Arc<dyn ConnectionProbe>,
    pub backups: Arc<dyn DatabaseBackups>,
    pub restore: Arc<dyn DatabaseRestore>,
    pub database_folders: Arc<dyn DatabaseFolders>,
    pub restart: Arc<dyn ApplicationRestart>,
    /// Paths and URL fragments a session must never read, copy or search: where credentials are kept.
    pub protected_credential_references: Vec<String>,
    /// `None` when the update check is turned off, so it never reaches the network.
    pub release_feed: Option<Arc<dyn ReleaseFeed>>,
}
