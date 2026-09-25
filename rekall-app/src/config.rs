//! The application's properties, read the way Spring Boot read `application.yaml`'s: a
//! `--name=value` command-line argument wins, then an environment variable in Spring's relaxed
//! form (`server.port` from `SERVER_PORT`, `rekall.claude.cli-path` from `REKALL_CLAUDE_CLIPATH`),
//! then the default the Java code declared.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use rekall_claude::ClaudeConfig;

/// Not 8080: see `application.yaml`. The MCP endpoint is registered with Claude Code as a fixed
/// URL, so a clash on a common port is a broken registration, not a retry.
pub const DEFAULT_PORT: u16 = 47355;

/// Where the database comes from when it is set by hand (`REKALL_DB_URL`,
/// `spring.datasource.url`) instead of by the registry in `~/.rekall/config.json`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DatabaseOverride {
    /// A SQLite file.
    File(PathBuf),
    /// A database that lives as long as the process.
    Memory,
}

impl DatabaseOverride {
    /// Accepts the JDBC URLs the Java build took, so a script that set one keeps working:
    /// `jdbc:h2:file:<base>[;options]` is the SQLite file `<base>.db` beside where H2 kept
    /// `<base>.mv.db`, and `jdbc:h2:mem:...` is in memory. `sqlite:<path>`, `sqlite::memory:` and a
    /// bare path are read as themselves.
    pub fn parse(url: &str) -> Option<Self> {
        let url = url.trim();
        if url.is_empty() {
            return None;
        }
        if url.starts_with("jdbc:h2:mem:") || url == "sqlite::memory:" || url == ":memory:" {
            return Some(Self::Memory);
        }
        if let Some(rest) = url.strip_prefix("jdbc:h2:file:") {
            let base = rest.split(';').next().unwrap_or(rest);
            return Some(Self::File(PathBuf::from(format!("{base}.db"))));
        }
        let path = url.strip_prefix("sqlite://").or_else(|| url.strip_prefix("sqlite:")).unwrap_or(url);
        let path = path.split('?').next().unwrap_or(path);
        Some(Self::File(PathBuf::from(path)))
    }
}

#[derive(Clone, Debug)]
pub struct BackupConfig {
    /// `rekall.backup.enabled`: the scheduled backups. A backup on demand works either way.
    pub enabled: bool,
    /// `rekall.backup.interval-hours`.
    pub interval_hours: i64,
    /// `rekall.backup.keep`.
    pub keep: usize,
}

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

/// Command-line arguments over environment variables, with Spring's relaxed names.
#[derive(Clone, Debug, Default)]
pub struct Properties {
    arguments: HashMap<String, String>,
    environment: HashMap<String, String>,
}

impl Properties {
    pub fn new(args: &[String], environment: HashMap<String, String>) -> Self {
        let mut arguments = HashMap::new();
        for arg in args {
            if let Some(rest) = arg.strip_prefix("--") {
                if let Some((name, value)) = rest.split_once('=') {
                    arguments.insert(name.to_string(), value.to_string());
                }
            }
        }
        Self { arguments, environment }
    }

    /// Properties set in code: what the tests use instead of `@DynamicPropertySource`.
    pub fn of(pairs: &[(&str, &str)]) -> Self {
        Self {
            arguments: pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
            environment: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<String> {
        if let Some(value) = self.arguments.get(name) {
            return Some(value.clone());
        }
        if let Some(value) = self.environment.get(name) {
            return Some(value.clone());
        }
        self.environment.get(&relaxed(name)).cloned()
    }

    fn number<T: std::str::FromStr>(&self, name: &str, default: T) -> T {
        self.get(name).and_then(|v| v.trim().parse().ok()).unwrap_or(default)
    }

    fn flag(&self, name: &str, default: bool) -> bool {
        match self.get(name).map(|v| v.trim().to_lowercase()) {
            Some(v) if matches!(v.as_str(), "true" | "on" | "yes" | "1") => true,
            Some(v) if matches!(v.as_str(), "false" | "off" | "no" | "0") => false,
            _ => default,
        }
    }
}

/// `rekall.claude.cli-path` -> `REKALL_CLAUDE_CLIPATH`: dots to underscores, dashes dropped.
fn relaxed(name: &str) -> String {
    name.replace('.', "_").replace('-', "").to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arguments_beat_the_environment_which_is_read_in_its_relaxed_form() {
        let environment = HashMap::from([
            ("SERVER_PORT".to_string(), "9000".to_string()),
            ("REKALL_CLAUDE_CLIPATH".to_string(), "/opt/claude".to_string()),
            ("REKALL_BACKUP_ENABLED".to_string(), "false".to_string()),
        ]);
        let properties = Properties::new(&["--server.port=9100".to_string()], environment);
        let config = AppConfig::from_sources(&properties);
        assert_eq!(config.port, 9100);
        assert_eq!(config.claude.cli_path.as_deref(), Some("/opt/claude"));
        assert!(!config.backup.enabled);
        assert_eq!(config.backup.keep, 10);
    }

    #[test]
    fn a_jdbc_url_from_the_java_build_names_the_sqlite_file_beside_the_h2_one() {
        assert_eq!(
            DatabaseOverride::parse("jdbc:h2:file:/data/demo/rekall;AUTO_SERVER=TRUE;DB_CLOSE_DELAY=-1"),
            Some(DatabaseOverride::File(PathBuf::from("/data/demo/rekall.db")))
        );
        assert_eq!(DatabaseOverride::parse("jdbc:h2:mem:rekall;DB_CLOSE_DELAY=-1"), Some(DatabaseOverride::Memory));
        assert_eq!(DatabaseOverride::parse("sqlite:/tmp/x.db"), Some(DatabaseOverride::File(PathBuf::from("/tmp/x.db"))));
        assert_eq!(DatabaseOverride::parse("  "), None);
    }
}
