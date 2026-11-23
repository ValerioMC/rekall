use rekall_store::task_decision_option;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecisionOptionView {
    pub label: String,
    pub effect: Option<String>,
    pub recommended: bool,
}

impl DecisionOptionView {
    pub fn of(option: &task_decision_option::Model) -> Self {
        Self { label: option.label.clone(), effect: option.effect.clone(), recommended: option.recommended }
    }
}
