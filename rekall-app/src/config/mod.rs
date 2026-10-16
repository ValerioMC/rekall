//! The application's properties: a `--name=value` command-line argument wins, then an environment
//! variable in relaxed form (`server.port` from `SERVER_PORT`, `rekall.claude.cli-path` from
//! `REKALL_CLAUDE_CLIPATH`), then the default declared in `AppConfig`.

mod app_config;

pub use app_config::{AppConfig, DEFAULT_PORT};
