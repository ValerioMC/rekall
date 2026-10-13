use serde::Deserialize;

/// Where a step was dragged to; both left out hands it back to the automatic layout.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct StepFlowPlaceRequest {
    pub x: Option<i32>,
    pub y: Option<i32>,
}
