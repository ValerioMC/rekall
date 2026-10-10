//! The `rekall.claude.*`, `rekall.terminal.*` and `rekall.run-queue.*` properties, with their
//! defaults.

use std::path::PathBuf;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct ClaudeConfig {
    /// `rekall.claude.cli-path`: overrides where `claude` is looked for.
    pub cli_path: Option<String>,
    /// `rekall.claude.usage-url`.
    pub usage_url: String,
    /// `rekall.terminal.shell`: overrides `$SHELL` for the login-shell environment.
    pub shell: Option<String>,
    /// `rekall.terminal.shell-environment-timeout-seconds`.
    pub shell_environment_timeout: Duration,
    /// `rekall.terminal.max-sessions`.
    pub max_sessions: usize,
    /// `rekall.terminal.idle-minutes`.
    pub idle_minutes: u64,
    /// `rekall.terminal.sweep-minutes`.
    pub sweep_minutes: u64,
    /// `rekall.terminal.scrollback-bytes`.
    pub scrollback_bytes: usize,
    /// `rekall.run-queue.tick-seconds`.
    pub run_queue_tick: Duration,
    /// `rekall.run-queue.settle-grace-seconds`.
    pub run_queue_settle_grace: Duration,
    /// `rekall.claude.transcripts-dir`: where Claude Code keeps its session transcripts.
    pub transcripts_dir: PathBuf,
    /// `rekall.claude.transcript-scan-seconds`: how often they are read for token usage.
    pub transcript_scan_every: Duration,
    /// The user's home.
    pub home: PathBuf,
}

impl Default for ClaudeConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_default();
        Self {
            cli_path: None,
            usage_url: "https://api.anthropic.com/api/oauth/usage".into(),
            shell: None,
            shell_environment_timeout: Duration::from_secs(10),
            max_sessions: 8,
            idle_minutes: 120,
            sweep_minutes: 5,
            scrollback_bytes: 131_072,
            run_queue_tick: Duration::from_secs(20),
            run_queue_settle_grace: Duration::from_secs(8),
            transcripts_dir: transcripts_under(&home),
            transcript_scan_every: Duration::from_secs(60),
            home,
        }
    }
}

/// Claude Code's own default for its session transcripts.
pub fn transcripts_under(home: &std::path::Path) -> PathBuf {
    home.join(".claude").join("projects")
}
