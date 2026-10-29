use std::sync::Arc;

use rekall_core::skill::SkillGroupService;

use crate::connection::CredentialBroker;
use crate::resources::ClaudeResourceService;
use crate::usage;

/// Everything this module serves, built over the shared services.
#[derive(Clone)]
pub struct ClaudeState {
    pub usage: Arc<dyn usage::UsageReader>,
    pub resources: ClaudeResourceService,
    pub skill_groups: SkillGroupService,
    /// Answers the credential helpers of the terminals.
    pub broker: Arc<CredentialBroker>,
}
