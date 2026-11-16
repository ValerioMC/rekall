use serde::Deserialize;

/// What a person has typed of a project key or a space key so far.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TargetSuggestionQuery {
    pub query: Option<String>,
}
