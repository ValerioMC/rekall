/// A Jira project or a Confluence space a binding's target can be filled with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetSuggestionView {
    pub value: String,
    pub name: String,
}
