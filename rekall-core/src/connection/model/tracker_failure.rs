/// Why a task manager did not take an issue. Nothing here quotes the credential.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TrackerFailure {
    /// The task manager answered 401 or 403 to the stored credential.
    Rejected { status: u16 },
    /// The network, TLS, or an answer the task manager explained with its own message.
    Failed { reason: String },
}
