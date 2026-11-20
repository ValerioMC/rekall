use serde::Deserialize;

/// The group whose contents are listed, and what has been typed of a name inside it.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GitlabChildrenQuery {
    pub parent: Option<String>,
    pub query: Option<String>,
}
