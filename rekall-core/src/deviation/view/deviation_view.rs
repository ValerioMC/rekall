use rekall_store::task_deviation;

/// A deviation as a context hands it over: the step it refers to and what changed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviationView {
    pub step_number: i32,
    pub step_title: Option<String>,
    pub body_markdown: String,
}

impl DeviationView {
    pub fn of(deviation: &task_deviation::Model, step_title: Option<String>) -> Self {
        Self { step_number: deviation.step_number, step_title, body_markdown: deviation.body_markdown.clone() }
    }
}
