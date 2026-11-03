use rekall_store::SessionKind;

/// The queue's settings: no ceiling when `ceiling_percent` is none, the account's model and effort
/// when those are, and terminal sessions unless the kind says otherwise.
#[derive(Clone, Debug, Default)]
pub struct RunQueueSettingsCommand {
    pub ceiling_percent: Option<i32>,
    pub skip_permissions: bool,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub session_kind: SessionKind,
    /// None keeps the queue working one task at a time.
    pub parallel_sessions: Option<i32>,
}
