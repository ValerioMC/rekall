use serde::Deserialize;

/// A step's visit budget; null lifts it.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct StepMaxVisitsRequest {
    pub max_visits: Option<i32>,
}
