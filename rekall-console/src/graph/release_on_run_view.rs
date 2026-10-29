use rekall_service::step::TaskStepView;

/// What running the task laid: the release graph's title, or none when nothing was laid, and the
/// task's whole checklist either way.
#[derive(Clone, Debug)]
pub struct ReleaseOnRunView {
    pub laid: Option<String>,
    pub steps: Vec<TaskStepView>,
}
