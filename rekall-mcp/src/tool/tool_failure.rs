use rekall_kernel::RekallError;

use crate::protocol::ToolError;

/// The hint a tool appends when a bare task label matched on more than one project.
pub(crate) const QUALIFY_WITH_PROJECT: &str = ". Qualify it with `project:<label>`.";

/// What each write tool did around its service call: an unknown anchor, an ambiguous one and a
/// broken rule are told to the session; anything else is an error.
pub(crate) fn told(error: RekallError, ambiguous_hint: &str) -> ToolError {
    match error {
        RekallError::UnknownAnchor(message) | RekallError::IllegalArgument(message) => ToolError::Failure(message),
        RekallError::AmbiguousAnchor { message, .. } => ToolError::Failure(format!("{message}{ambiguous_hint}")),
        other => ToolError::Other(other),
    }
}
