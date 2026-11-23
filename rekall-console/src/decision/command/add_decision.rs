/// A decision the person writes by hand: answered from the start, on the task or on one of its steps.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AddDecision {
    pub topic: Option<String>,
    pub question: Option<String>,
    pub answer: Option<String>,
    pub step_number: Option<i32>,
}
