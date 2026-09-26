use rekall_common::RekallError;

/// Why a tool call did not produce its answer.
#[derive(Debug)]
pub enum ToolError {
    /// `ToolFailure`: the session asked for something this tool will not do; it is told why.
    Failure(String),
    /// `IllegalArgumentException`: an argument broke a rule; it is told why, too.
    Illegal(String),
    /// Anything else: a JSON-RPC error.
    Other(RekallError),
}

impl From<RekallError> for ToolError {
    fn from(error: RekallError) -> Self {
        match error {
            RekallError::IllegalArgument(message) => Self::Illegal(message),
            other => Self::Other(other),
        }
    }
}
