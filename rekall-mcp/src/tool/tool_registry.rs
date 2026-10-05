use std::sync::Arc;

use rekall_service::Services;

use crate::protocol::McpTool;

use super::{CommitReferenceTool, ContextTool, DiagramTool, NoteTool, ProjectSectionTool, StepProposalTool, StepStateTool, WrapupTool};

/// Every tool, in the order `tools/list` announces them.
pub fn all(services: &Services) -> Vec<Arc<dyn McpTool>> {
    vec![
        Arc::new(CommitReferenceTool::new(services.commit_references.clone())),
        Arc::new(ContextTool::new(services.context.clone())),
        Arc::new(NoteTool::new(services.notes.clone())),
        Arc::new(StepProposalTool::new(services.steps.clone())),
        Arc::new(StepStateTool::new(services.auto_commit.clone(), services.steps.clone())),
        Arc::new(WrapupTool::new(services.auto_commit.clone(), services.wrapups.clone())),
        Arc::new(DiagramTool::new(services.diagrams.clone())),
        Arc::new(ProjectSectionTool::new(services.sections.clone())),
    ]
}
