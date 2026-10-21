use rekall_service::connection::Secret;

/// Why a helper's request was refused, which decides the status the endpoint answers with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// The token is not this terminal's, or the request did not come from this machine.
    Forbidden,
    /// No such live terminal, connection or stored credential.
    Missing,
    /// The helper asked for a shape this connection's kind does not have.
    WrongPurpose,
    /// The store or the log failed; nothing is handed out unlogged.
    Failed,
}

/// The endpoint's answer: the credential in the shape the helper reads, or why not, in words the
/// helper prints. The credential stays a `Secret`, so this prints as `***` in a log.
#[derive(Debug)]
pub enum HandOut {
    Granted { body: Secret, content_type: &'static str },
    Refused { refusal: Refusal, message: String },
}

impl HandOut {
    pub fn refused(refusal: Refusal, message: impl Into<String>) -> Self {
        Self::Refused { refusal, message: message.into() }
    }
}
