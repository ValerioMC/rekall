use rekall_kernel::Instant;
use rekall_store::task_decision_detail;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecisionDetailView {
    pub text: String,
    pub added_at: Instant,
}

impl DecisionDetailView {
    pub fn of(detail: &task_decision_detail::Model) -> Self {
        Self { text: detail.text.clone(), added_at: detail.added_at }
    }
}
