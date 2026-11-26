use std::sync::Arc;

use rekall_core::backup::{DatabaseBackups, DatabaseRestore};
use rekall_core::chat::ChatPipe;
use rekall_core::claude::{ClaudeCodeSetup, ClaudeResources, TerminalHost, TerminalPipe, UsageReader};
use rekall_core::connection::{ConfluenceSpaces, ConnectionProbe, CredentialDesk, GitlabNamespaces, JiraIssues, SecretStore, TaskTrackerService};
use rekall_core::guard::GuardInstaller;
use rekall_core::picture::PictureStore;
use rekall_core::queue::QueueRunner;
use rekall_core::settings::{ApplicationRestart, DatabaseFolders};
use rekall_core::version::ReleaseFeed;
use rekall_core::worktree::TaskWorktreeService;

/// What the routes reach on the machine, each behind a port of `rekall-core`. `rekall-app` builds
/// the implementations and hands them over here.
#[derive(Clone)]
pub struct ApiPorts {
    pub secrets: Arc<dyn SecretStore>,
    pub jira: Arc<dyn JiraIssues>,
    pub confluence: Arc<dyn ConfluenceSpaces>,
    pub gitlab: Arc<dyn GitlabNamespaces>,
    pub terminals: Arc<dyn TerminalHost>,
    pub terminal_pipe: Arc<dyn TerminalPipe>,
    pub chat_pipe: Arc<dyn ChatPipe>,
    pub queue_runner: Arc<dyn QueueRunner>,
    pub usage: Arc<dyn UsageReader>,
    pub codex: Arc<dyn rekall_core::codex::CodexClient>,
    pub claude_folder: Arc<dyn ClaudeResources>,
    pub claude_setup: Arc<dyn ClaudeCodeSetup>,
    pub claude_guard: Arc<dyn GuardInstaller>,
    pub codex_guard: Arc<dyn GuardInstaller>,
    pub credential_desk: Arc<dyn CredentialDesk>,
    pub connection_probe: Arc<dyn ConnectionProbe>,
    pub backups: Arc<dyn DatabaseBackups>,
    pub restore: Arc<dyn DatabaseRestore>,
    pub database_folders: Arc<dyn DatabaseFolders>,
    pub pictures: Arc<dyn PictureStore>,
    pub restart: Arc<dyn ApplicationRestart>,
    /// The writer of tasks to their project's task manager, which the runner also drives.
    pub task_tracker: TaskTrackerService,
    /// The task worktrees, over the git port the run queue also uses.
    pub task_worktrees: TaskWorktreeService,
    /// Paths and URL fragments a session must never read, copy or search: where credentials are kept.
    pub protected_credential_references: Vec<String>,
    /// `None` when the update check is turned off, so it never reaches the network.
    pub release_feed: Option<Arc<dyn ReleaseFeed>>,
}
