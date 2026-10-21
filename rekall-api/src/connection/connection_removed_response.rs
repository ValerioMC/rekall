use serde::Serialize;

/// What deleting a connection took with it.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionRemovedResponse {
    pub unbound_projects: usize,
}
