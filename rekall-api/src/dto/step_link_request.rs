use serde::Deserialize;

/// A route as the console sends it: on create both fields are required, on edit either may be
/// left out to keep it.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct StepLinkRequest {
    pub to_step_id: Option<String>,
    pub outcome: Option<String>,
}
