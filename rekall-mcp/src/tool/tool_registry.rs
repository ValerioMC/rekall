use std::sync::Arc;

use rekall_service::Services;

use crate::protocol::McpTool;

use super::{CommitReferenceTool, ContextTool, NoteTool, StepProposalTool, StepStateTool, WrapupTool};

/// Every tool, in the order the Java server listed them.
pub fn all(services: &Services) -> Vec<Arc<dyn McpTool>> {
    vec![
        Arc::new(CommitReferenceTool::new(services.clone())),
        Arc::new(ContextTool::new(services.clone())),
        Arc::new(NoteTool::new(services.clone())),
        Arc::new(StepProposalTool::new(services.clone())),
        Arc::new(StepStateTool::new(services.clone())),
        Arc::new(WrapupTool::new(services.clone())),
    ]
}
