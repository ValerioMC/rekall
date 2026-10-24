use serde::Deserialize;

/// A route out of a step being written: the outcome it leaves by, and the key of the step it
/// leads to, or none for the end of the flow.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GraphRouteRequest {
    pub outcome: Option<String>,
    pub to_key: Option<String>,
}
