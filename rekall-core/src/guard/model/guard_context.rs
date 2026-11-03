/// The facts a decision needs, read by the caller from Rekall.
/// `protected_credential_references` are paths or URL fragments that must never appear in a tool call.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuardContext {
    pub auto_commit: bool,
    pub running_step_number: Option<u32>,
    pub protected_credential_references: Vec<String>,
}
