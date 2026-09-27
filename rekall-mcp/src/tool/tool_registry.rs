use std::sync::Arc;

use rekall_service::Services;

use crate::protocol::McpTool;

use super::{CommitReferenceTool, ContextTool, DiagramTool, NoteTool, StepProposalTool, StepStateTool, WrapupTool};

/// Every tool: the Java server's six in its order, then `rekall_diagram`.
pub fn all(services: &Services) -> Vec<Arc<dyn McpTool>> {
    vec![
        Arc::new(CommitReferenceTool::new(services.clone())),
        Arc::new(ContextTool::new(services.clone())),
        Arc::new(NoteTool::new(services.clone())),
        Arc::new(StepProposalTool::new(services.clone())),
        Arc::new(StepStateTool::new(services.clone())),
        Arc::new(WrapupTool::new(services.clone())),
        Arc::new(DiagramTool::new(services.clone())),
    ]
}
