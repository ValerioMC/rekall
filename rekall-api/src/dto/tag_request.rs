use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TagRequest {
    pub name: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
}
