//! The six tools: `rekall_context` reads; `rekall_wrapup`, `rekall_step`, `rekall_record_commit`,
//! `rekall_propose_step` and `rekall_note` write, each one narrow thing.

mod anchor;
mod arguments;
mod commit_reference;
mod context;
mod note;
mod step_proposal;
mod step_state;
mod wrapup;

use std::sync::Arc;

use rekall_common::RekallError;
use rekall_service::Services;

pub use anchor::{Anchor, AnchoredTask};
pub use arguments::Arguments;
pub use commit_reference::CommitReferenceTool;
pub use context::ContextTool;
pub use note::NoteTool;
pub use step_proposal::StepProposalTool;
pub use step_state::StepStateTool;
pub use wrapup::WrapupTool;

use crate::protocol::{McpTool, ToolError};

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

/// What each write tool did around its service call: an unknown anchor, an ambiguous one and a
/// broken rule are told to the session; anything else is an error.
fn told(error: RekallError, ambiguous_hint: &str) -> ToolError {
    match error {
        RekallError::UnknownAnchor(message) | RekallError::IllegalArgument(message) => ToolError::Failure(message),
        RekallError::AmbiguousAnchor { message, .. } => ToolError::Failure(format!("{message}{ambiguous_hint}")),
        other => ToolError::Other(other),
    }
}

const QUALIFY_WITH_PROJECT: &str = ". Qualify it with `project:<label>`.";
