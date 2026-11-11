/// What Rekall pushes of a task to the project's task manager: its title and its markdown description.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrackerIssue {
    pub title: String,
    pub description: String,
}
