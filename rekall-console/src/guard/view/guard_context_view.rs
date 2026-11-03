use super::running_step_view::RunningStepView;

/// `project_anchor` is `None` when no project's folder holds the session's folder: the commit
/// rules do not apply then, the credential references still do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuardContextView {
    pub project_anchor: Option<String>,
    pub auto_commit: bool,
    pub running_step: Option<RunningStepView>,
    pub protected_credential_references: Vec<String>,
}
