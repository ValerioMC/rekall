/// A Jira project as Rekall reads it, enough to offer it as a binding's target.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JiraProject {
    pub key: String,
    pub name: String,
}
