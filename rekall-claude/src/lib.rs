//! Claude Code hosted in Rekall: the interactive `claude` TUI in a pseudo-terminal per task,
//! piped to the console over one WebSocket; the account's usage read for the meter; and the run
//! queue that works through tasks one terminal at a time. Nothing about a terminal is persisted.

pub mod cli;
pub mod config;
pub mod credentials;
pub mod login_shell;
pub mod pty;
pub mod queue;
pub mod socket;
pub mod terminal;
pub mod usage;

use std::sync::Arc;

use axum::Router;
use rekall_api::ApiState;

pub use config::ClaudeConfig;

/// Everything this module serves, built over the shared services.
#[derive(Clone)]
pub struct ClaudeState {
    pub api: ApiState,
    pub terminals: Arc<pty::PtyTerminalManager>,
    pub usage: Arc<dyn usage::UsageReader>,
    pub queue: queue::RunQueueService,
    pub runner: queue::RunQueueRunner,
}

pub fn router(state: ClaudeState) -> Router {
    Router::new()
        .merge(terminal::routes())
        .merge(socket::routes())
        .merge(usage::routes())
        .merge(queue::routes())
        .with_state(state)
}
