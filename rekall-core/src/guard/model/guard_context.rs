use super::guard_late_changes::GuardLateChanges;
use super::guard_running_step::GuardRunningStep;
use super::guard_merge::GuardMerge;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuardContext {
    pub auto_commit: bool,
    pub merge: Option<GuardMerge>,
    pub running_step: Option<GuardRunningStep>,
    pub late_changes: Option<GuardLateChanges>,
    pub protected_credential_references: Vec<String>,
}
