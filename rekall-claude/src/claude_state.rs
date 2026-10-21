use std::sync::Arc;

use crate::connection::CredentialBroker;
use crate::resources::{ClaudeResourceService, SkillGroupService};
use crate::{pty, queue, usage};

/// Everything this module serves, built over the shared services.
#[derive(Clone)]
pub struct ClaudeState {
    pub terminals: Arc<pty::PtyTerminalManager>,
    pub usage: Arc<dyn usage::UsageReader>,
    pub queue: queue::RunQueueService,
    pub runner: queue::RunQueueRunner,
    pub resources: ClaudeResourceService,
    pub skill_groups: SkillGroupService,
    /// Answers the credential helpers of the terminals.
    pub broker: Arc<CredentialBroker>,
}
