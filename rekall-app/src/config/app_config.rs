use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use rekall_claude::ClaudeConfig;

use super::{BackupConfig, DatabaseOverride, Properties};

/// Not 8080: see `application.yaml`. The MCP endpoint is registered with Claude Code as a fixed
/// URL, so a clash on a common port is a broken registration, not a retry.
pub const DEFAULT_PORT: u16 = 47355;

#[derive(Clone, Debug)]
pub struct AppConfig {
    /// `server.port`; 0 picks a free one, as in the tests.
    pub port: u16,
    /// `rekall.home`: where `config.json` lives.
    pub home: PathBuf,
    /// `REKALL_DB_URL` / `spring.datasource.url`.
    pub database: Option<DatabaseOverride>,
    /// `rekall.security.remote-access`.
    pub remote_access: bool,
    pub backup: BackupConfig,
    pub claude: ClaudeConfig,
    /// `rekall.ui.dist`: a folder to serve the console from instead of the bundle built into the
    /// binary, for working on the UI without rebuilding the server.
    pub ui_dist: Option<PathBuf>,
    /// The user's home: `System.getProperty("user.home")`.
    pub user_home: PathBuf,
    /// Where `./data` resolves: the working directory the server was started from.
    pub working_dir: PathBuf,
    /// How long the web server waits for requests in flight on shutdown
    /// (`spring.lifecycle.timeout-per-shutdown-phase`).
    pub shutdown_timeout: Duration,
}

impl AppConfig {
    /// The properties of this process: its arguments and its environment.
    pub fn from_process(args: &[String]) -> Self {
        let environment: HashMap<String, String> = std::env::vars().collect();
        Self::from_sources(&Properties::new(args, environment))
    }

    pub fn from_sources(properties: &Properties) -> Self {
        let user_home = properties
            .get("user.home")
            .map(PathBuf::from)
            .or_else(dirs::home_dir)
            .unwrap_or_default();
        let defaults = ClaudeConfig { home: user_home.clone(), ..ClaudeConfig::default() };
        let claude = ClaudeConfig {
            cli_path: properties.get("rekall.claude.cli-path").filter(|p| !p.trim().is_empty()),
            usage_url: properties.get("rekall.claude.usage-url").unwrap_or(defaults.usage_url.clone()),
            shell: properties.get("rekall.terminal.shell").filter(|s| !s.trim().is_empty()),
            shell_environment_timeout: Duration::from_secs(
                properties.number("rekall.terminal.shell-environment-timeout-seconds", 10u64),
            ),
            max_sessions: properties.number("rekall.terminal.max-sessions", defaults.max_sessions),
            idle_minutes: properties.number("rekall.terminal.idle-minutes", defaults.idle_minutes),
            sweep_minutes: properties.number("rekall.terminal.sweep-minutes", defaults.sweep_minutes),
            scrollback_bytes: properties.number("rekall.terminal.scrollback-bytes", defaults.scrollback_bytes),
            run_queue_tick: Duration::from_secs(properties.number("rekall.run-queue.tick-seconds", 20u64)),
            run_queue_settle_grace: Duration::from_secs(properties.number("rekall.run-queue.settle-grace-seconds", 8u64)),
            home: user_home.clone(),
        };
        let database = properties
            .get("REKALL_DB_URL")
            .or_else(|| properties.get("spring.datasource.url"))
            .and_then(|url| DatabaseOverride::parse(&url));
        Self {
            port: properties.number("server.port", DEFAULT_PORT),
            home: properties.get("rekall.home").map(PathBuf::from).unwrap_or_else(|| user_home.join(".rekall")),
            database,
            remote_access: properties.flag("rekall.security.remote-access", false),
            backup: BackupConfig {
                enabled: properties.flag("rekall.backup.enabled", true),
                interval_hours: properties.number("rekall.backup.interval-hours", 24i64),
                keep: properties.number("rekall.backup.keep", 10usize),
            },
            claude,
            ui_dist: properties.get("rekall.ui.dist").filter(|p| !p.trim().is_empty()).map(PathBuf::from),
            user_home,
            working_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            shutdown_timeout: Duration::from_secs(5),
        }
    }

    /// The URL the MCP endpoint is registered under: `ClaudeCodeController`'s.
    pub fn mcp_endpoint(&self, served_port: u16) -> String {
        let port = if served_port > 0 { served_port } else { DEFAULT_PORT };
        format!("http://localhost:{port}/mcp")
    }
}

#[cfg(test)]
#[path = "../../tests/unit/config/app_config_tests.rs"]
mod tests;
