/// A step a session wants on the task it creates: a title and, optionally, the detail behind it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StepDraft {
    pub title: String,
    pub detail: Option<String>,
}
