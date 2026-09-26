use std::sync::Arc;

use rekall_api::ApiState;

use crate::{pty, queue, usage};

/// Everything this module serves, built over the shared services.
#[derive(Clone)]
pub struct ClaudeState {
    pub api: ApiState,
    pub terminals: Arc<pty::PtyTerminalManager>,
    pub usage: Arc<dyn usage::UsageReader>,
    pub queue: queue::RunQueueService,
    pub runner: queue::RunQueueRunner,
}
