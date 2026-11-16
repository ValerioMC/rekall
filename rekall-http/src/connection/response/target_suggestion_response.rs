use rekall_console::connection::TargetSuggestionView;
use serde::Serialize;

/// A Jira project or a Confluence space a binding's target can be filled with.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetSuggestionResponse {
    pub value: String,
    pub name: String,
}

impl From<TargetSuggestionView> for TargetSuggestionResponse {
    fn from(view: TargetSuggestionView) -> Self {
        Self { value: view.value, name: view.name }
    }
}
