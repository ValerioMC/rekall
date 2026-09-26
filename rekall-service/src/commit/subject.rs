/// What the commit is for: the step or task claimed, as a title, where it lives, and the markdown
/// that describes it (a step's detail or the task's wrapup), which may be absent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Subject {
    pub title: String,
    pub anchor: String,
    pub step_number: Option<i32>,
    pub description: Option<String>,
}
