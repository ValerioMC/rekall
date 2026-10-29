use rekall_kernel::RekallError;

/// A method's handler failed: a broken rule is `-32602` (invalid params), anything else `-32603`.
pub(super) enum Failure {
    Illegal(String),
    Other(RekallError),
}

impl From<RekallError> for Failure {
    fn from(error: RekallError) -> Self {
        match error {
            RekallError::IllegalArgument(message) => Self::Illegal(message),
            other => Self::Other(other),
        }
    }
}
