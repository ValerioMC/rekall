use rekall_console::workingset::WorkingSetView;
use serde::Serialize;

use crate::workingset::{OpenTaskResponse, PlaceResponse};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkingSetResponse {
    pub tasks: Vec<OpenTaskResponse>,
    pub places: Vec<PlaceResponse>,
}

impl From<WorkingSetView> for WorkingSetResponse {
    fn from(view: WorkingSetView) -> Self {
        Self { tasks: view.tasks.into_iter().map(Into::into).collect(), places: view.places.into_iter().map(Into::into).collect() }
    }
}
