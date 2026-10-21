use serde::Deserialize;

/// A connection to create or edit. On an edit, a field left out keeps what the connection has; on
/// a create, a label left out is made from the title. `kind` is fixed once created.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ConnectionRequest {
    pub kind: Option<String>,
    pub label: Option<String>,
    pub title: Option<String>,
    pub base_url: Option<String>,
}
