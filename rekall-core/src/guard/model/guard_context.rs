use super::guard_running_step::GuardRunningStep;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuardContext {
    pub auto_commit: bool,
    pub running_step: Option<GuardRunningStep>,
    pub protected_credential_references: Vec<String>,
}
